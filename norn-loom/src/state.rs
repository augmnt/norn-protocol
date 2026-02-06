use std::collections::HashMap;

use norn_crypto::hash::blake3_hash;
use norn_types::primitives::{Hash, LoomId};

/// In-memory key-value state for a single loom contract.
#[derive(Debug, Clone)]
pub struct LoomState {
    /// The loom this state belongs to.
    pub loom_id: LoomId,
    /// Key-value data.
    pub data: HashMap<Vec<u8>, Vec<u8>>,
}

impl LoomState {
    /// Create a new empty loom state.
    pub fn new(loom_id: LoomId) -> Self {
        Self {
            loom_id,
            data: HashMap::new(),
        }
    }

    /// Get a value by key.
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    /// Set a key-value pair.
    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    /// Delete a key. Returns `true` if the key existed.
    pub fn delete(&mut self, key: &[u8]) -> bool {
        self.data.remove(key).is_some()
    }

    /// Compute a deterministic hash of the entire state.
    ///
    /// Keys are sorted lexicographically. Each (key, value) pair is
    /// borsh-encoded, and the concatenation of all encoded pairs is hashed
    /// with BLAKE3.
    pub fn compute_hash(&self) -> Hash {
        let mut sorted_keys: Vec<&Vec<u8>> = self.data.keys().collect();
        sorted_keys.sort();

        let mut buf = Vec::new();
        for key in sorted_keys {
            let value = &self.data[key];
            // Encode key length + key + value length + value using borsh.
            let pair: (Vec<u8>, Vec<u8>) = (key.clone(), value.clone());
            if let Ok(encoded) = borsh::to_vec(&pair) {
                buf.extend_from_slice(&encoded);
            }
        }

        blake3_hash(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set_delete() {
        let mut state = LoomState::new([0u8; 32]);

        // Initially empty.
        assert!(state.get(b"key").is_none());

        // Set a value.
        state.set(b"key".to_vec(), b"value".to_vec());
        assert_eq!(state.get(b"key"), Some(b"value".as_ref()));

        // Overwrite.
        state.set(b"key".to_vec(), b"new_value".to_vec());
        assert_eq!(state.get(b"key"), Some(b"new_value".as_ref()));

        // Delete.
        assert!(state.delete(b"key"));
        assert!(state.get(b"key").is_none());

        // Delete non-existent returns false.
        assert!(!state.delete(b"key"));
    }

    #[test]
    fn test_hash_determinism() {
        let mut state_a = LoomState::new([1u8; 32]);
        state_a.set(b"x".to_vec(), b"1".to_vec());
        state_a.set(b"y".to_vec(), b"2".to_vec());

        // Same data, inserted in different order.
        let mut state_b = LoomState::new([1u8; 32]);
        state_b.set(b"y".to_vec(), b"2".to_vec());
        state_b.set(b"x".to_vec(), b"1".to_vec());

        assert_eq!(state_a.compute_hash(), state_b.compute_hash());
    }

    #[test]
    fn test_hash_changes_on_mutation() {
        let mut state = LoomState::new([0u8; 32]);
        state.set(b"a".to_vec(), b"1".to_vec());
        let hash1 = state.compute_hash();

        state.set(b"a".to_vec(), b"2".to_vec());
        let hash2 = state.compute_hash();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_empty_state_hash() {
        let state = LoomState::new([0u8; 32]);
        let hash = state.compute_hash();
        // Should be a valid 32-byte hash (BLAKE3 of empty data).
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_differs_by_key() {
        let mut state_a = LoomState::new([0u8; 32]);
        state_a.set(b"key_a".to_vec(), b"value".to_vec());

        let mut state_b = LoomState::new([0u8; 32]);
        state_b.set(b"key_b".to_vec(), b"value".to_vec());

        assert_ne!(state_a.compute_hash(), state_b.compute_hash());
    }
}
