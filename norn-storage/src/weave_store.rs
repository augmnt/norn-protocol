use borsh::BorshDeserialize;

use norn_types::primitives::Hash;
use norn_types::weave::{WeaveBlock, WeaveState};

use crate::error::StorageError;
use crate::traits::KvStore;

const WEAVE_BLOCK_HEIGHT_PREFIX: &[u8] = b"weave:block:height:";
const WEAVE_BLOCK_HASH_PREFIX: &[u8] = b"weave:block:hash:";
const WEAVE_STATE_KEY: &[u8] = b"weave:state";

/// Storage layer for weave blocks and global weave state.
pub struct WeaveStore<S: KvStore> {
    store: S,
}

impl<S: KvStore> WeaveStore<S> {
    /// Create a new WeaveStore wrapping the given KvStore.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Save a weave block. Stores it by both height and hash for dual-index lookup.
    pub fn save_block(&self, block: &WeaveBlock) -> Result<(), StorageError> {
        let value = borsh::to_vec(block).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;

        // Save by height
        let mut height_key =
            Vec::with_capacity(WEAVE_BLOCK_HEIGHT_PREFIX.len() + std::mem::size_of::<u64>());
        height_key.extend_from_slice(WEAVE_BLOCK_HEIGHT_PREFIX);
        height_key.extend_from_slice(&block.height.to_be_bytes());
        self.store.put(&height_key, &value)?;

        // Save by hash
        let mut hash_key = Vec::with_capacity(WEAVE_BLOCK_HASH_PREFIX.len() + 32);
        hash_key.extend_from_slice(WEAVE_BLOCK_HASH_PREFIX);
        hash_key.extend_from_slice(&block.hash);
        self.store.put(&hash_key, &value)?;

        Ok(())
    }

    /// Load a weave block by height.
    pub fn load_block(&self, height: u64) -> Result<Option<WeaveBlock>, StorageError> {
        let mut key =
            Vec::with_capacity(WEAVE_BLOCK_HEIGHT_PREFIX.len() + std::mem::size_of::<u64>());
        key.extend_from_slice(WEAVE_BLOCK_HEIGHT_PREFIX);
        key.extend_from_slice(&height.to_be_bytes());

        match self.store.get(&key)? {
            Some(bytes) => {
                let block = WeaveBlock::try_from_slice(&bytes).map_err(|e| {
                    StorageError::DeserializationError {
                        reason: e.to_string(),
                    }
                })?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Load a weave block by its hash.
    pub fn load_block_by_hash(&self, hash: &Hash) -> Result<Option<WeaveBlock>, StorageError> {
        let mut key = Vec::with_capacity(WEAVE_BLOCK_HASH_PREFIX.len() + 32);
        key.extend_from_slice(WEAVE_BLOCK_HASH_PREFIX);
        key.extend_from_slice(hash);

        match self.store.get(&key)? {
            Some(bytes) => {
                let block = WeaveBlock::try_from_slice(&bytes).map_err(|e| {
                    StorageError::DeserializationError {
                        reason: e.to_string(),
                    }
                })?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Save the global weave state.
    pub fn save_weave_state(&self, state: &WeaveState) -> Result<(), StorageError> {
        let value = borsh::to_vec(state).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(WEAVE_STATE_KEY, &value)
    }

    /// Load the global weave state.
    pub fn load_weave_state(&self) -> Result<Option<WeaveState>, StorageError> {
        match self.store.get(WEAVE_STATE_KEY)? {
            Some(bytes) => {
                let state = WeaveState::try_from_slice(&bytes).map_err(|e| {
                    StorageError::DeserializationError {
                        reason: e.to_string(),
                    }
                })?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// Get the latest block height from the weave state.
    pub fn latest_height(&self) -> Result<Option<u64>, StorageError> {
        match self.load_weave_state()? {
            Some(state) => Ok(Some(state.height)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryStore;
    use norn_types::weave::FeeState;

    fn make_store() -> WeaveStore<MemoryStore> {
        WeaveStore::new(MemoryStore::new())
    }

    fn sample_block(height: u64, hash: Hash) -> WeaveBlock {
        WeaveBlock {
            height,
            hash,
            prev_hash: [0u8; 32],
            commitments_root: [1u8; 32],
            registrations_root: [2u8; 32],
            anchors_root: [3u8; 32],
            commitments: vec![],
            registrations: vec![],
            anchors: vec![],
            name_registrations: vec![],
            name_registrations_root: [0u8; 32],
            fraud_proofs: vec![],
            fraud_proofs_root: [0u8; 32],
            transfers: vec![],
            transfers_root: [0u8; 32],
            token_definitions: vec![],
            token_definitions_root: [0u8; 32],
            token_mints: vec![],
            token_mints_root: [0u8; 32],
            token_burns: vec![],
            token_burns_root: [0u8; 32],
            loom_deploys: vec![],
            loom_deploys_root: [0u8; 32],
            timestamp: 1000 + height,
            proposer: [4u8; 32],
            validator_signatures: vec![],
        }
    }

    fn sample_weave_state(height: u64) -> WeaveState {
        WeaveState {
            height,
            latest_hash: [10u8; 32],
            threads_root: [11u8; 32],
            thread_count: 5,
            fee_state: FeeState {
                base_fee: 100,
                fee_multiplier: 1000,
                epoch_fees: 50000,
            },
        }
    }

    #[test]
    fn test_block_save_load_by_height() {
        let ws = make_store();
        let block = sample_block(0, [42u8; 32]);
        ws.save_block(&block).unwrap();

        let loaded = ws.load_block(0).unwrap();
        assert_eq!(loaded, Some(block));
    }

    #[test]
    fn test_block_save_load_by_hash() {
        let ws = make_store();
        let hash = [42u8; 32];
        let block = sample_block(0, hash);
        ws.save_block(&block).unwrap();

        let loaded = ws.load_block_by_hash(&hash).unwrap();
        assert_eq!(loaded, Some(block));
    }

    #[test]
    fn test_block_not_found() {
        let ws = make_store();
        assert_eq!(ws.load_block(999).unwrap(), None);
        assert_eq!(ws.load_block_by_hash(&[0u8; 32]).unwrap(), None);
    }

    #[test]
    fn test_multiple_blocks() {
        let ws = make_store();
        for i in 0..5u64 {
            let mut hash = [0u8; 32];
            hash[0] = i as u8;
            let block = sample_block(i, hash);
            ws.save_block(&block).unwrap();
        }

        for i in 0..5u64 {
            let loaded = ws.load_block(i).unwrap().unwrap();
            assert_eq!(loaded.height, i);
        }
    }

    #[test]
    fn test_weave_state_save_load() {
        let ws = make_store();
        let state = sample_weave_state(100);
        ws.save_weave_state(&state).unwrap();

        let loaded = ws.load_weave_state().unwrap();
        assert_eq!(loaded, Some(state));
    }

    #[test]
    fn test_weave_state_not_found() {
        let ws = make_store();
        assert_eq!(ws.load_weave_state().unwrap(), None);
    }

    #[test]
    fn test_latest_height() {
        let ws = make_store();
        assert_eq!(ws.latest_height().unwrap(), None);

        let state = sample_weave_state(42);
        ws.save_weave_state(&state).unwrap();
        assert_eq!(ws.latest_height().unwrap(), Some(42));
    }

    #[test]
    fn test_weave_state_overwrite() {
        let ws = make_store();
        let state1 = sample_weave_state(1);
        ws.save_weave_state(&state1).unwrap();

        let state2 = sample_weave_state(2);
        ws.save_weave_state(&state2).unwrap();

        let loaded = ws.load_weave_state().unwrap().unwrap();
        assert_eq!(loaded.height, 2);
    }
}
