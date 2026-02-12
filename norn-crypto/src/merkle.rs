use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};

use borsh::{BorshDeserialize, BorshSerialize};
use norn_types::error::NornError;
use norn_types::primitives::Hash;
use serde::{Deserialize, Serialize};

use crate::hash::blake3_hash;

/// The zero hash used for empty nodes.
pub const EMPTY_HASH: Hash = [0u8; 32];

/// The depth of the sparse Merkle tree (256 bits = 32 bytes key space).
pub const TREE_DEPTH: usize = 256;

/// A proof of inclusion (or non-inclusion) in a sparse Merkle tree.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The key being proved.
    pub key: Hash,
    /// The value at the key (empty vec for non-inclusion).
    pub value: Vec<u8>,
    /// Sibling hashes from depth 0 to TREE_DEPTH-1.
    pub siblings: Vec<Hash>,
}

// ─── FxHash-style hasher for pre-hashed keys ────────────────────────────────

/// A fast non-cryptographic hasher using multiply-rotate-xor mixing.
/// Works well for blake3-derived Hash keys which are already uniformly
/// distributed. Faster than SipHash for small-to-medium HashMaps.
#[derive(Default)]
struct FxHasher(u64);

const FX_SEED: u64 = 0x517cc1b727220a95;

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            let val = u64::from_ne_bytes(chunk.try_into().unwrap());
            self.0 = (self.0.rotate_left(5) ^ val).wrapping_mul(FX_SEED);
        }
        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            let mut buf = [0u8; 8];
            buf[..remainder.len()].copy_from_slice(remainder);
            let val = u64::from_ne_bytes(buf);
            self.0 = (self.0.rotate_left(5) ^ val).wrapping_mul(FX_SEED);
        }
    }

    #[inline]
    fn write_usize(&mut self, _: usize) {
        // Ignored — length prefix is constant for fixed-size [u8; 32] keys
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Default)]
struct FxBuildHasher;

impl BuildHasher for FxBuildHasher {
    type Hasher = FxHasher;

    #[inline]
    fn build_hasher(&self) -> FxHasher {
        FxHasher(0)
    }
}

type FastMap<K, V> = HashMap<K, V, FxBuildHasher>;

#[inline]
fn new_fast_map<K, V>() -> FastMap<K, V> {
    HashMap::with_hasher(FxBuildHasher)
}

// ─── Sparse Merkle Tree ─────────────────────────────────────────────────────

/// In-memory sparse Merkle tree with O(1) root queries and O(TREE_DEPTH)
/// incremental path updates.
///
/// Uses a layered structure: one HashMap per depth level. Each HashMap
/// maps truncated key prefixes to cached node hashes. This keeps individual
/// HashMaps small (~N entries where N is the number of data keys) for
/// better cache behavior than a single massive HashMap.
pub struct SparseMerkleTree {
    /// Stored key-value pairs.
    data: FastMap<Hash, Vec<u8>>,
    /// Cached node hashes per depth level. `nodes[d]` maps a truncated
    /// prefix (with bits d..255 zeroed) to the cached hash at depth d.
    /// Index 0 is unused (root is stored separately), indices 1..=256
    /// correspond to depth levels 1..=256 (where 256 is the leaf level).
    nodes: Vec<FastMap<Hash, Hash>>,
    /// The current root hash, maintained incrementally.
    cached_root: Hash,
}

impl SparseMerkleTree {
    /// Create a new empty sparse Merkle tree.
    pub fn new() -> Self {
        let mut nodes = Vec::with_capacity(TREE_DEPTH + 1);
        for _ in 0..=TREE_DEPTH {
            nodes.push(new_fast_map());
        }
        Self {
            data: new_fast_map(),
            nodes,
            cached_root: EMPTY_HASH,
        }
    }

    /// Get the current root hash. O(1).
    pub fn root(&mut self) -> Hash {
        if self.data.is_empty() {
            EMPTY_HASH
        } else {
            self.cached_root
        }
    }

    /// Insert a key-value pair and incrementally update the root.
    pub fn insert(&mut self, key: Hash, value: Vec<u8>) {
        self.data.insert(key, value);
        self.update_path(&key);
    }

    /// Batch-insert multiple key-value pairs. All data is loaded before any
    /// path updates, ensuring correct sibling hashes for overlapping paths.
    pub fn insert_batch(&mut self, entries: Vec<(Hash, Vec<u8>)>) {
        if entries.is_empty() {
            return;
        }
        for (key, value) in &entries {
            self.data.insert(*key, value.clone());
        }
        for (key, _) in &entries {
            self.update_path(key);
        }
    }

    /// Get a value by key.
    pub fn get(&self, key: &Hash) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    /// Remove a key from the tree and update the root.
    pub fn remove(&mut self, key: &Hash) -> bool {
        let removed = self.data.remove(key).is_some();
        if removed {
            self.update_path(key);
        }
        removed
    }

    /// Generate a Merkle proof for a key. O(TREE_DEPTH) lookups.
    pub fn prove(&mut self, key: &Hash) -> MerkleProof {
        let value = self.data.get(key).cloned().unwrap_or_default();
        let mut siblings = Vec::with_capacity(TREE_DEPTH);

        for depth in 0..TREE_DEPTH {
            let mut sibling_prefix = truncate_key(key, depth + 1);
            flip_bit(&mut sibling_prefix, depth);
            let sibling_hash = self.nodes[depth + 1]
                .get(&sibling_prefix)
                .copied()
                .unwrap_or(EMPTY_HASH);
            siblings.push(sibling_hash);
        }

        MerkleProof {
            key: *key,
            value,
            siblings,
        }
    }

    /// Verify a Merkle proof against a given root.
    pub fn verify_proof(root: &Hash, proof: &MerkleProof) -> Result<(), NornError> {
        if proof.siblings.len() != TREE_DEPTH {
            return Err(NornError::MerkleProofInvalid);
        }

        let mut current = if proof.value.is_empty() {
            EMPTY_HASH
        } else {
            let value_hash = blake3_hash(&proof.value);
            hash_leaf(&proof.key, &value_hash)
        };

        for depth in (0..TREE_DEPTH).rev() {
            let bit = get_bit(&proof.key, depth);
            let sibling = &proof.siblings[depth];
            if current == EMPTY_HASH && *sibling == EMPTY_HASH {
                current = EMPTY_HASH;
            } else {
                current = if bit == 0 {
                    hash_internal(&current, sibling)
                } else {
                    hash_internal(sibling, &current)
                };
            }
        }

        if current == *root {
            Ok(())
        } else {
            Err(NornError::MerkleProofInvalid)
        }
    }

    /// Incrementally update all cached node hashes along the path from a
    /// leaf to the root.
    ///
    /// Uses incremental prefix computation via `clear_bit_at` to avoid
    /// allocating a new prefix array at each depth level.
    ///
    /// Complexity: O(TREE_DEPTH) = O(256) hash computations + HashMap ops.
    fn update_path(&mut self, key: &Hash) {
        let leaf_hash = if let Some(value) = self.data.get(key) {
            let value_hash = blake3_hash(value);
            hash_leaf(key, &value_hash)
        } else {
            EMPTY_HASH
        };

        let mut current_hash = leaf_hash;
        let mut our_prefix = *key;

        for depth in (0..TREE_DEPTH).rev() {
            let bit = get_bit(key, depth);
            let depth_idx = depth + 1;

            // Store our node at depth+1
            self.nodes[depth_idx].insert(our_prefix, current_hash);

            // Look up sibling at depth+1
            let mut sibling_prefix = our_prefix;
            flip_bit(&mut sibling_prefix, depth);
            let sibling_hash = self.nodes[depth_idx]
                .get(&sibling_prefix)
                .copied()
                .unwrap_or(EMPTY_HASH);

            // Compute parent hash
            let (left, right) = if bit == 0 {
                (current_hash, sibling_hash)
            } else {
                (sibling_hash, current_hash)
            };

            current_hash = if left == EMPTY_HASH && right == EMPTY_HASH {
                EMPTY_HASH
            } else {
                hash_internal(&left, &right)
            };

            // Prepare prefix for next (shallower) level
            clear_bit_at(&mut our_prefix, depth);
        }

        self.cached_root = current_hash;
    }
}

impl Default for SparseMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Key helpers ────────────────────────────────────────────────────────────

/// Truncate a key to its first `depth` bits, zeroing all bits from `depth` onwards.
#[inline]
fn truncate_key(key: &Hash, depth: usize) -> Hash {
    if depth >= TREE_DEPTH {
        return *key;
    }
    let mut result = [0u8; 32];
    let full_bytes = depth / 8;
    let remaining_bits = depth % 8;
    if full_bytes > 0 {
        result[..full_bytes].copy_from_slice(&key[..full_bytes]);
    }
    if remaining_bits > 0 && full_bytes < 32 {
        let mask = !((1u8 << (8 - remaining_bits)) - 1);
        result[full_bytes] = key[full_bytes] & mask;
    }
    result
}

// ─── Bit helpers ────────────────────────────────────────────────────────────

/// Flip a single bit at position `pos` in a hash (MSB-first ordering).
#[inline]
fn flip_bit(key: &mut Hash, pos: usize) {
    let byte_idx = pos / 8;
    let bit_idx = 7 - (pos % 8);
    key[byte_idx] ^= 1 << bit_idx;
}

/// Clear a single bit at position `pos` in a hash (MSB-first ordering).
#[inline]
fn clear_bit_at(key: &mut Hash, pos: usize) {
    let byte_idx = pos / 8;
    let bit_idx = 7 - (pos % 8);
    key[byte_idx] &= !(1 << bit_idx);
}

// ─── Hash helpers ───────────────────────────────────────────────────────────

/// Hash a leaf node: H(0x00 || key || value_hash).
pub fn hash_leaf(key: &Hash, value_hash: &Hash) -> Hash {
    let mut data = [0u8; 65];
    data[0] = 0x00;
    data[1..33].copy_from_slice(key);
    data[33..65].copy_from_slice(value_hash);
    blake3_hash(&data)
}

/// Hash an internal node: H(0x01 || left || right).
pub fn hash_internal(left: &Hash, right: &Hash) -> Hash {
    let mut data = [0u8; 65];
    data[0] = 0x01;
    data[1..33].copy_from_slice(left);
    data[33..65].copy_from_slice(right);
    blake3_hash(&data)
}

/// Get bit at position `depth` from a hash key (MSB first).
pub fn get_bit(key: &Hash, depth: usize) -> u8 {
    let byte_index = depth / 8;
    let bit_index = 7 - (depth % 8);
    if byte_index < 32 {
        (key[byte_index] >> bit_index) & 1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let mut tree = SparseMerkleTree::new();
        assert_eq!(tree.root(), EMPTY_HASH);
    }

    #[test]
    fn test_insert_and_get() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"key1");
        let value = b"value1".to_vec();
        tree.insert(key, value.clone());
        assert_eq!(tree.get(&key), Some(value.as_slice()));
    }

    #[test]
    fn test_insert_changes_root() {
        let mut tree = SparseMerkleTree::new();
        let old_root = tree.root();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec());
        assert_ne!(tree.root(), old_root);
    }

    #[test]
    fn test_get_nonexistent() {
        let tree = SparseMerkleTree::new();
        let key = blake3_hash(b"nonexistent");
        assert_eq!(tree.get(&key), None);
    }

    #[test]
    fn test_update_value() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec());
        tree.insert(key, b"value2".to_vec());
        assert_eq!(tree.get(&key), Some(b"value2".as_slice()));
    }

    #[test]
    fn test_remove() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec());
        assert!(tree.remove(&key));
        assert_eq!(tree.get(&key), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"nonexistent");
        assert!(!tree.remove(&key));
    }

    #[test]
    fn test_remove_restores_empty_root() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec());
        tree.remove(&key);
        assert_eq!(tree.root(), EMPTY_HASH);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut tree = SparseMerkleTree::new();
        for i in 0..10u8 {
            let key = blake3_hash(&[i]);
            tree.insert(key, vec![i; 10]);
        }
        for i in 0..10u8 {
            let key = blake3_hash(&[i]);
            assert_eq!(tree.get(&key), Some(vec![i; 10].as_slice()));
        }
    }

    #[test]
    fn test_proof_inclusion() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"key1");
        let value = b"value1".to_vec();
        tree.insert(key, value.clone());

        let root = tree.root();
        let proof = tree.prove(&key);
        assert_eq!(proof.key, key);
        assert_eq!(proof.value, value);
        assert_eq!(proof.siblings.len(), TREE_DEPTH);
        assert!(SparseMerkleTree::verify_proof(&root, &proof).is_ok());
    }

    #[test]
    fn test_proof_non_inclusion() {
        let mut tree = SparseMerkleTree::new();
        let root = tree.root();
        let key = blake3_hash(b"nonexistent");
        let proof = tree.prove(&key);
        assert!(proof.value.is_empty());
        assert!(SparseMerkleTree::verify_proof(&root, &proof).is_ok());
    }

    #[test]
    fn test_proof_invalid_root() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec());
        let proof = tree.prove(&key);
        let wrong_root = blake3_hash(b"wrong");
        assert!(SparseMerkleTree::verify_proof(&wrong_root, &proof).is_err());
    }

    #[test]
    fn test_proof_after_multiple_inserts() {
        let mut tree = SparseMerkleTree::new();
        let keys_values: Vec<(Hash, Vec<u8>)> =
            (0..5u8).map(|i| (blake3_hash(&[i]), vec![i; 20])).collect();

        for (key, value) in &keys_values {
            tree.insert(*key, value.clone());
        }

        let root = tree.root();
        for (key, value) in &keys_values {
            let proof = tree.prove(key);
            assert_eq!(proof.value, *value);
            assert!(SparseMerkleTree::verify_proof(&root, &proof).is_ok());
        }
    }

    #[test]
    fn test_update_value_changes_root() {
        let mut tree = SparseMerkleTree::new();
        let key = blake3_hash(b"key1");
        tree.insert(key, b"value1".to_vec());
        let root1 = tree.root();
        tree.insert(key, b"value2".to_vec());
        let root2 = tree.root();
        assert_ne!(root1, root2);
    }

    #[test]
    fn test_proof_after_removal() {
        let mut tree = SparseMerkleTree::new();
        let key1 = blake3_hash(b"key1");
        let key2 = blake3_hash(b"key2");
        tree.insert(key1, b"value1".to_vec());
        tree.insert(key2, b"value2".to_vec());
        tree.remove(&key1);

        let root = tree.root();
        let proof = tree.prove(&key2);
        assert!(SparseMerkleTree::verify_proof(&root, &proof).is_ok());
        let proof1 = tree.prove(&key1);
        assert!(proof1.value.is_empty());
        assert!(SparseMerkleTree::verify_proof(&root, &proof1).is_ok());
    }

    #[test]
    fn test_incremental_root_consistency() {
        let keys: Vec<(Hash, Vec<u8>)> = (0..20u8)
            .map(|i| (blake3_hash(&[i]), vec![i; 16]))
            .collect();

        let mut incremental = SparseMerkleTree::new();
        for (key, value) in &keys {
            incremental.insert(*key, value.clone());

            let mut fresh = SparseMerkleTree::new();
            for (k2, v2) in &keys[..=(keys.iter().position(|x| x.0 == *key).unwrap())] {
                fresh.insert(*k2, v2.clone());
            }
            assert_eq!(
                incremental.root(),
                fresh.root(),
                "roots diverged after inserting key index {}",
                keys.iter().position(|x| x.0 == *key).unwrap()
            );
        }
    }

    #[test]
    fn test_insert_batch_matches_sequential() {
        let entries: Vec<(Hash, Vec<u8>)> = (0..50u8)
            .map(|i| (blake3_hash(&[i]), vec![i; 16]))
            .collect();

        let mut sequential = SparseMerkleTree::new();
        for (key, value) in &entries {
            sequential.insert(*key, value.clone());
        }

        let mut batched = SparseMerkleTree::new();
        batched.insert_batch(entries.clone());

        assert_eq!(
            sequential.root(),
            batched.root(),
            "batch insert must produce the same root as sequential inserts"
        );

        for (key, value) in &entries {
            assert_eq!(batched.get(key), Some(value.as_slice()));
        }
    }

    #[test]
    fn test_insert_batch_empty() {
        let mut tree = SparseMerkleTree::new();
        tree.insert_batch(vec![]);
        assert_eq!(tree.root(), EMPTY_HASH);
    }

    #[test]
    fn test_insert_batch_then_individual() {
        let batch_entries: Vec<(Hash, Vec<u8>)> =
            (0..10u8).map(|i| (blake3_hash(&[i]), vec![i; 8])).collect();

        let mut tree = SparseMerkleTree::new();
        tree.insert_batch(batch_entries.clone());

        let extra_key = blake3_hash(&[99u8]);
        tree.insert(extra_key, vec![99; 8]);

        let mut sequential = SparseMerkleTree::new();
        for (key, value) in &batch_entries {
            sequential.insert(*key, value.clone());
        }
        sequential.insert(extra_key, vec![99; 8]);

        assert_eq!(tree.root(), sequential.root());
    }

    #[test]
    fn test_truncate_key() {
        let key = [0xFF; 32];
        assert_eq!(truncate_key(&key, 0), [0u8; 32]);
        let mut expected = [0u8; 32];
        expected[0] = 0xFF;
        assert_eq!(truncate_key(&key, 8), expected);
        let mut expected = [0u8; 32];
        expected[0] = 0xF0;
        assert_eq!(truncate_key(&key, 4), expected);
        assert_eq!(truncate_key(&key, 256), key);
    }
}
