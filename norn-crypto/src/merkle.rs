use std::collections::HashMap;

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

/// In-memory sparse Merkle tree.
///
/// Uses a simple approach: store key-value pairs and compute the root
/// lazily using hash caching.
pub struct SparseMerkleTree {
    /// Stored key-value pairs.
    data: HashMap<Hash, Vec<u8>>,
    /// Cached internal node hashes: (depth, prefix_bits) -> hash.
    /// Invalidated on mutation.
    cache: HashMap<(usize, Vec<u8>), Hash>,
    /// Whether the cache is valid.
    cache_valid: bool,
}

impl SparseMerkleTree {
    /// Create a new empty sparse Merkle tree.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            cache: HashMap::new(),
            cache_valid: false,
        }
    }

    /// Get the current root hash.
    pub fn root(&mut self) -> Hash {
        if self.data.is_empty() {
            return EMPTY_HASH;
        }
        if !self.cache_valid {
            self.cache.clear();
            self.cache_valid = true;
        }
        self.compute_node(0, &[])
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: Hash, value: Vec<u8>) {
        self.data.insert(key, value);
        self.cache_valid = false;
    }

    /// Get a value by key.
    pub fn get(&self, key: &Hash) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    /// Remove a key from the tree.
    pub fn remove(&mut self, key: &Hash) -> bool {
        let removed = self.data.remove(key).is_some();
        if removed {
            self.cache_valid = false;
        }
        removed
    }

    /// Generate a Merkle proof for a key.
    pub fn prove(&mut self, key: &Hash) -> MerkleProof {
        if !self.cache_valid {
            self.cache.clear();
            self.cache_valid = true;
            // Force computation of the full tree
            if !self.data.is_empty() {
                self.compute_node(0, &[]);
            }
        }

        let value = self.data.get(key).cloned().unwrap_or_default();
        let mut siblings = Vec::with_capacity(TREE_DEPTH);

        for depth in 0..TREE_DEPTH {
            let bit = get_bit(key, depth);
            // Get the sibling's prefix (same as ours but with the opposite bit at this depth)
            let mut prefix = get_prefix(key, depth);
            let sibling_bit = if bit == 0 { 1u8 } else { 0u8 };
            prefix.push(sibling_bit);

            let sibling_hash = self.compute_node(depth + 1, &prefix);
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

        // Start from the leaf
        let mut current = if proof.value.is_empty() {
            EMPTY_HASH
        } else {
            let value_hash = blake3_hash(&proof.value);
            hash_leaf(&proof.key, &value_hash)
        };

        // Walk from the leaf (depth TREE_DEPTH-1) up to the root (depth 0)
        for depth in (0..TREE_DEPTH).rev() {
            let bit = get_bit(&proof.key, depth);
            let sibling = &proof.siblings[depth];
            // If both children are empty, the parent is also empty
            // (matches compute_node behavior)
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

    /// Compute the hash of a node at the given depth with the given path prefix.
    fn compute_node(&mut self, depth: usize, prefix: &[u8]) -> Hash {
        // Check cache
        let cache_key = (depth, prefix.to_vec());
        if let Some(&cached) = self.cache.get(&cache_key) {
            return cached;
        }

        let result = if depth == TREE_DEPTH {
            // Leaf level — find if any key matches this exact path
            self.data
                .iter()
                .find(|(k, _)| key_matches_prefix(k, prefix))
                .map(|(k, v)| {
                    let value_hash = blake3_hash(v);
                    hash_leaf(k, &value_hash)
                })
                .unwrap_or(EMPTY_HASH)
        } else {
            // Internal level — check if any keys exist under this prefix
            let has_keys = self.data.keys().any(|k| key_matches_prefix(k, prefix));
            if !has_keys {
                EMPTY_HASH
            } else {
                let mut left_prefix = prefix.to_vec();
                left_prefix.push(0);
                let mut right_prefix = prefix.to_vec();
                right_prefix.push(1);

                let left = self.compute_node(depth + 1, &left_prefix);
                let right = self.compute_node(depth + 1, &right_prefix);

                if left == EMPTY_HASH && right == EMPTY_HASH {
                    EMPTY_HASH
                } else {
                    hash_internal(&left, &right)
                }
            }
        };

        self.cache.insert(cache_key, result);
        result
    }
}

impl Default for SparseMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Hash helpers ────────────────────────────────────────────────────────────

pub fn hash_leaf(key: &Hash, value_hash: &Hash) -> Hash {
    let mut data = Vec::with_capacity(65);
    data.push(0x00); // Leaf prefix
    data.extend_from_slice(key);
    data.extend_from_slice(value_hash);
    blake3_hash(&data)
}

pub fn hash_internal(left: &Hash, right: &Hash) -> Hash {
    let mut data = Vec::with_capacity(65);
    data.push(0x01); // Internal prefix
    data.extend_from_slice(left);
    data.extend_from_slice(right);
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
}
