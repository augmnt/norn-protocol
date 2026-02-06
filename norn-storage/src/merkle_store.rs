use std::collections::HashMap;

use norn_crypto::hash::blake3_hash;
use norn_crypto::merkle::{
    get_bit, hash_internal, hash_leaf, MerkleProof, SparseMerkleTree, EMPTY_HASH, TREE_DEPTH,
};
use norn_types::primitives::Hash;

use crate::error::StorageError;
use crate::traits::KvStore;

const MERKLE_DATA_PREFIX: &[u8] = b"merkle:data:";
const MERKLE_ROOT_KEY: &[u8] = b"merkle:root";

/// A persistent Merkle tree that mirrors SparseMerkleTree's API but persists
/// all data to a KvStore. On insert/remove, the root is recomputed from all
/// stored key-value pairs using the same hash functions as SparseMerkleTree.
pub struct PersistentMerkleTree<S: KvStore> {
    store: S,
    /// Cached root hash. Invalidated (set to None) on mutations.
    cached_root: Option<Hash>,
}

impl<S: KvStore> PersistentMerkleTree<S> {
    /// Create a new PersistentMerkleTree wrapping the given KvStore.
    pub fn new(store: S) -> Self {
        Self {
            store,
            cached_root: None,
        }
    }

    /// Build the storage key for a data entry.
    fn data_key(key: &Hash) -> Vec<u8> {
        let mut k = Vec::with_capacity(MERKLE_DATA_PREFIX.len() + 32);
        k.extend_from_slice(MERKLE_DATA_PREFIX);
        k.extend_from_slice(key);
        k
    }

    /// Insert a key-value pair into the tree.
    pub fn insert(&mut self, key: Hash, value: Vec<u8>) -> Result<(), StorageError> {
        let storage_key = Self::data_key(&key);
        self.store.put(&storage_key, &value)?;
        // Invalidate cached root
        self.cached_root = None;
        self.store.delete(MERKLE_ROOT_KEY)?;
        Ok(())
    }

    /// Get a value by key.
    pub fn get(&self, key: &Hash) -> Result<Option<Vec<u8>>, StorageError> {
        let storage_key = Self::data_key(key);
        self.store.get(&storage_key)
    }

    /// Remove a key from the tree. Returns true if the key existed.
    pub fn remove(&mut self, key: &Hash) -> Result<bool, StorageError> {
        let storage_key = Self::data_key(key);
        let existed = self.store.exists(&storage_key)?;
        if existed {
            self.store.delete(&storage_key)?;
            // Invalidate cached root
            self.cached_root = None;
            self.store.delete(MERKLE_ROOT_KEY)?;
        }
        Ok(existed)
    }

    /// Load all data entries from the store.
    fn load_all_data(&self) -> Result<HashMap<Hash, Vec<u8>>, StorageError> {
        let entries = self.store.prefix_scan(MERKLE_DATA_PREFIX)?;
        let mut data = HashMap::new();
        for (key_bytes, value) in entries {
            if key_bytes.len() == MERKLE_DATA_PREFIX.len() + 32 {
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&key_bytes[MERKLE_DATA_PREFIX.len()..]);
                data.insert(hash, value);
            }
        }
        Ok(data)
    }

    /// Compute the root hash by loading all data and building the tree
    /// in-memory, exactly mirroring SparseMerkleTree's logic.
    fn compute_root(&self) -> Result<Hash, StorageError> {
        let data = self.load_all_data()?;
        if data.is_empty() {
            return Ok(EMPTY_HASH);
        }

        let mut cache: HashMap<(usize, Vec<u8>), Hash> = HashMap::new();
        let root = compute_node(0, &[], &data, &mut cache);
        Ok(root)
    }

    /// Get the current root hash. Uses cached root if available,
    /// otherwise computes and caches it.
    pub fn root(&mut self) -> Result<Hash, StorageError> {
        if let Some(root) = self.cached_root {
            return Ok(root);
        }

        // Try to load from store
        if let Some(root_bytes) = self.store.get(MERKLE_ROOT_KEY)? {
            if root_bytes.len() == 32 {
                let mut root = [0u8; 32];
                root.copy_from_slice(&root_bytes);
                self.cached_root = Some(root);
                return Ok(root);
            }
        }

        // Compute from data
        let root = self.compute_root()?;
        self.store.put(MERKLE_ROOT_KEY, &root)?;
        self.cached_root = Some(root);
        Ok(root)
    }

    /// Generate a Merkle proof for a key.
    /// Loads all data and computes the proof in-memory.
    pub fn prove(&mut self, key: &Hash) -> Result<MerkleProof, StorageError> {
        let data = self.load_all_data()?;

        // Build the full tree cache
        let mut cache: HashMap<(usize, Vec<u8>), Hash> = HashMap::new();
        if !data.is_empty() {
            compute_node(0, &[], &data, &mut cache);
        }

        let value = data.get(key).cloned().unwrap_or_default();
        let mut siblings = Vec::with_capacity(TREE_DEPTH);

        for depth in 0..TREE_DEPTH {
            let bit = get_bit(key, depth);
            let mut prefix = get_prefix(key, depth);
            let sibling_bit = if bit == 0 { 1u8 } else { 0u8 };
            prefix.push(sibling_bit);

            let sibling_hash = if data.is_empty() {
                EMPTY_HASH
            } else {
                compute_node_cached(depth + 1, &prefix, &data, &mut cache)
            };
            siblings.push(sibling_hash);
        }

        Ok(MerkleProof {
            key: *key,
            value,
            siblings,
        })
    }

    /// Verify a Merkle proof against a given root.
    /// Delegates to SparseMerkleTree::verify_proof.
    pub fn verify_proof(root: &Hash, proof: &MerkleProof) -> Result<(), StorageError> {
        SparseMerkleTree::verify_proof(root, proof).map_err(|e| StorageError::ReadError {
            reason: e.to_string(),
        })
    }
}

/// Get the first `depth` bits of a key as a Vec<u8> of 0s and 1s.
fn get_prefix(key: &Hash, depth: usize) -> Vec<u8> {
    (0..depth).map(|d| get_bit(key, d)).collect()
}

/// Check if a key matches a given bit prefix.
fn key_matches_prefix(key: &Hash, prefix: &[u8]) -> bool {
    for (i, &bit) in prefix.iter().enumerate() {
        if get_bit(key, i) != bit {
            return false;
        }
    }
    true
}

/// Recursively compute a node hash, mirroring SparseMerkleTree::compute_node.
fn compute_node(
    depth: usize,
    prefix: &[u8],
    data: &HashMap<Hash, Vec<u8>>,
    cache: &mut HashMap<(usize, Vec<u8>), Hash>,
) -> Hash {
    let cache_key = (depth, prefix.to_vec());
    if let Some(&cached) = cache.get(&cache_key) {
        return cached;
    }

    let result = if depth == TREE_DEPTH {
        data.iter()
            .find(|(k, _)| key_matches_prefix(k, prefix))
            .map(|(k, v)| {
                let value_hash = blake3_hash(v);
                hash_leaf(k, &value_hash)
            })
            .unwrap_or(EMPTY_HASH)
    } else {
        let has_keys = data.keys().any(|k| key_matches_prefix(k, prefix));
        if !has_keys {
            EMPTY_HASH
        } else {
            let mut left_prefix = prefix.to_vec();
            left_prefix.push(0);
            let mut right_prefix = prefix.to_vec();
            right_prefix.push(1);

            let left = compute_node(depth + 1, &left_prefix, data, cache);
            let right = compute_node(depth + 1, &right_prefix, data, cache);

            if left == EMPTY_HASH && right == EMPTY_HASH {
                EMPTY_HASH
            } else {
                hash_internal(&left, &right)
            }
        }
    };

    cache.insert(cache_key, result);
    result
}

/// Compute a node hash using the cache, computing if not cached.
fn compute_node_cached(
    depth: usize,
    prefix: &[u8],
    data: &HashMap<Hash, Vec<u8>>,
    cache: &mut HashMap<(usize, Vec<u8>), Hash>,
) -> Hash {
    compute_node(depth, prefix, data, cache)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryStore;
    use std::sync::Arc;

    fn make_tree() -> PersistentMerkleTree<MemoryStore> {
        PersistentMerkleTree::new(MemoryStore::new())
    }

    #[test]
    fn test_empty_tree_root() {
        let mut tree = make_tree();
        assert_eq!(tree.root().unwrap(), EMPTY_HASH);
    }

    #[test]
    fn test_insert_and_get() {
        let mut tree = make_tree();
        let key = blake3_hash(b"key1");
        let value = b"value1".to_vec();
        tree.insert(key, value.clone()).unwrap();
        assert_eq!(tree.get(&key).unwrap(), Some(value));
    }

    #[test]
    fn test_insert_changes_root() {
        let mut tree = make_tree();
        let old_root = tree.root().unwrap();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec()).unwrap();
        assert_ne!(tree.root().unwrap(), old_root);
    }

    #[test]
    fn test_remove() {
        let mut tree = make_tree();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec()).unwrap();
        assert!(tree.remove(&key).unwrap());
        assert_eq!(tree.get(&key).unwrap(), None);
        assert_eq!(tree.root().unwrap(), EMPTY_HASH);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut tree = make_tree();
        let key = blake3_hash(b"nope");
        assert!(!tree.remove(&key).unwrap());
    }

    #[test]
    fn test_root_matches_in_memory_tree() {
        let mut persistent = make_tree();
        let mut in_memory = SparseMerkleTree::new();

        let keys_values: Vec<(Hash, Vec<u8>)> =
            (0..5u8).map(|i| (blake3_hash(&[i]), vec![i; 20])).collect();

        for (key, value) in &keys_values {
            persistent.insert(*key, value.clone()).unwrap();
            in_memory.insert(*key, value.clone());
        }

        assert_eq!(persistent.root().unwrap(), in_memory.root());
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let mut tree = make_tree();
        let key = blake3_hash(b"key1");
        let value = b"value1".to_vec();
        tree.insert(key, value.clone()).unwrap();

        let root = tree.root().unwrap();
        let proof = tree.prove(&key).unwrap();
        assert_eq!(proof.key, key);
        assert_eq!(proof.value, value);
        assert_eq!(proof.siblings.len(), TREE_DEPTH);

        PersistentMerkleTree::<MemoryStore>::verify_proof(&root, &proof).unwrap();
    }

    #[test]
    fn test_proof_non_inclusion() {
        let mut tree = make_tree();
        let root = tree.root().unwrap();
        let key = blake3_hash(b"nonexistent");
        let proof = tree.prove(&key).unwrap();
        assert!(proof.value.is_empty());
        PersistentMerkleTree::<MemoryStore>::verify_proof(&root, &proof).unwrap();
    }

    #[test]
    fn test_proof_after_multiple_inserts() {
        let mut tree = make_tree();
        let keys_values: Vec<(Hash, Vec<u8>)> =
            (0..5u8).map(|i| (blake3_hash(&[i]), vec![i; 20])).collect();

        for (key, value) in &keys_values {
            tree.insert(*key, value.clone()).unwrap();
        }

        let root = tree.root().unwrap();
        for (key, value) in &keys_values {
            let proof = tree.prove(key).unwrap();
            assert_eq!(proof.value, *value);
            PersistentMerkleTree::<MemoryStore>::verify_proof(&root, &proof).unwrap();
        }
    }

    #[test]
    fn test_persistence_across_restart() {
        // Create a shared store, insert data, compute root, then create a
        // new PersistentMerkleTree over the same store and verify data persists.
        let store = Arc::new(MemoryStore::new());

        let key = blake3_hash(b"persist_key");
        let value = b"persist_value".to_vec();

        let root1 = {
            let mut tree = PersistentMerkleTree::new(Arc::clone(&store));
            tree.insert(key, value.clone()).unwrap();
            tree.root().unwrap()
        };

        // "Restart": create a new tree over the same store
        let mut tree2 = PersistentMerkleTree::new(Arc::clone(&store));
        assert_eq!(tree2.get(&key).unwrap(), Some(value.clone()));

        let root2 = tree2.root().unwrap();
        assert_eq!(root1, root2);

        // Proof should also work
        let proof = tree2.prove(&key).unwrap();
        assert_eq!(proof.value, value);
        PersistentMerkleTree::<Arc<MemoryStore>>::verify_proof(&root2, &proof).unwrap();
    }

    #[test]
    fn test_proof_matches_in_memory_tree() {
        let mut persistent = make_tree();
        let mut in_memory = SparseMerkleTree::new();

        let key = blake3_hash(b"key1");
        let value = b"value1".to_vec();

        persistent.insert(key, value.clone()).unwrap();
        in_memory.insert(key, value.clone());

        let p_root = persistent.root().unwrap();
        let m_root = in_memory.root();
        assert_eq!(p_root, m_root);

        let p_proof = persistent.prove(&key).unwrap();
        let m_proof = in_memory.prove(&key);

        assert_eq!(p_proof.key, m_proof.key);
        assert_eq!(p_proof.value, m_proof.value);
        assert_eq!(p_proof.siblings, m_proof.siblings);
    }
}
