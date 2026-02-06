use borsh::BorshDeserialize;

use norn_types::knot::Knot;
use norn_types::primitives::ThreadId;
use norn_types::thread::{ThreadHeader, ThreadState};

use crate::error::StorageError;
use crate::traits::KvStore;

const THREAD_HEADER_PREFIX: &[u8] = b"thread:header:";
const THREAD_STATE_PREFIX: &[u8] = b"thread:state:";
const THREAD_KNOTS_PREFIX: &[u8] = b"thread:knots:";

/// Storage layer for thread headers, states, and uncommitted knots.
pub struct ThreadStore<S: KvStore> {
    store: S,
}

impl<S: KvStore> ThreadStore<S> {
    /// Create a new ThreadStore wrapping the given KvStore.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Build a key by concatenating a prefix with a thread ID.
    fn make_key(prefix: &[u8], thread_id: &ThreadId) -> Vec<u8> {
        let mut key = Vec::with_capacity(prefix.len() + thread_id.len());
        key.extend_from_slice(prefix);
        key.extend_from_slice(thread_id);
        key
    }

    /// Save a thread header.
    pub fn save_header(
        &self,
        thread_id: &ThreadId,
        header: &ThreadHeader,
    ) -> Result<(), StorageError> {
        let key = Self::make_key(THREAD_HEADER_PREFIX, thread_id);
        let value = borsh::to_vec(header).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    /// Load a thread header by thread ID.
    pub fn load_header(&self, thread_id: &ThreadId) -> Result<Option<ThreadHeader>, StorageError> {
        let key = Self::make_key(THREAD_HEADER_PREFIX, thread_id);
        match self.store.get(&key)? {
            Some(bytes) => {
                let header = ThreadHeader::try_from_slice(&bytes).map_err(|e| {
                    StorageError::DeserializationError {
                        reason: e.to_string(),
                    }
                })?;
                Ok(Some(header))
            }
            None => Ok(None),
        }
    }

    /// Save a thread state.
    pub fn save_state(
        &self,
        thread_id: &ThreadId,
        state: &ThreadState,
    ) -> Result<(), StorageError> {
        let key = Self::make_key(THREAD_STATE_PREFIX, thread_id);
        let value = borsh::to_vec(state).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    /// Load a thread state by thread ID.
    pub fn load_state(&self, thread_id: &ThreadId) -> Result<Option<ThreadState>, StorageError> {
        let key = Self::make_key(THREAD_STATE_PREFIX, thread_id);
        match self.store.get(&key)? {
            Some(bytes) => {
                let state = ThreadState::try_from_slice(&bytes).map_err(|e| {
                    StorageError::DeserializationError {
                        reason: e.to_string(),
                    }
                })?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// Save uncommitted knots for a thread.
    pub fn save_uncommitted_knots(
        &self,
        thread_id: &ThreadId,
        knots: &[Knot],
    ) -> Result<(), StorageError> {
        let key = Self::make_key(THREAD_KNOTS_PREFIX, thread_id);
        let value =
            borsh::to_vec(&knots.to_vec()).map_err(|e| StorageError::SerializationError {
                reason: e.to_string(),
            })?;
        self.store.put(&key, &value)
    }

    /// Load uncommitted knots for a thread.
    pub fn load_uncommitted_knots(&self, thread_id: &ThreadId) -> Result<Vec<Knot>, StorageError> {
        let key = Self::make_key(THREAD_KNOTS_PREFIX, thread_id);
        match self.store.get(&key)? {
            Some(bytes) => {
                let knots = Vec::<Knot>::try_from_slice(&bytes).map_err(|e| {
                    StorageError::DeserializationError {
                        reason: e.to_string(),
                    }
                })?;
                Ok(knots)
            }
            None => Ok(Vec::new()),
        }
    }

    /// List all thread IDs by scanning the header prefix.
    pub fn list_threads(&self) -> Result<Vec<ThreadId>, StorageError> {
        let results = self.store.prefix_scan(THREAD_HEADER_PREFIX)?;
        let mut thread_ids = Vec::new();
        for (key, _) in results {
            if key.len() == THREAD_HEADER_PREFIX.len() + 20 {
                let mut id = [0u8; 20];
                id.copy_from_slice(&key[THREAD_HEADER_PREFIX.len()..]);
                thread_ids.push(id);
            }
        }
        Ok(thread_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryStore;
    use norn_types::knot::*;
    use norn_types::primitives::NATIVE_TOKEN_ID;

    fn make_store() -> ThreadStore<MemoryStore> {
        ThreadStore::new(MemoryStore::new())
    }

    fn sample_header(thread_id: ThreadId) -> ThreadHeader {
        ThreadHeader {
            thread_id,
            owner: [2u8; 32],
            version: 1,
            state_hash: [3u8; 32],
            last_knot_hash: [0u8; 32],
            prev_header_hash: [0u8; 32],
            timestamp: 1000,
            signature: [5u8; 64],
        }
    }

    fn sample_state() -> ThreadState {
        let mut state = ThreadState::new();
        state.credit(NATIVE_TOKEN_ID, 1_000_000).unwrap();
        state.credit([1u8; 32], 500).unwrap();
        state
    }

    fn sample_knot() -> Knot {
        Knot {
            id: [10u8; 32],
            knot_type: KnotType::Transfer,
            timestamp: 2000,
            expiry: None,
            before_states: vec![ParticipantState {
                thread_id: [1u8; 20],
                pubkey: [2u8; 32],
                version: 0,
                state_hash: [3u8; 32],
            }],
            after_states: vec![ParticipantState {
                thread_id: [1u8; 20],
                pubkey: [2u8; 32],
                version: 1,
                state_hash: [4u8; 32],
            }],
            payload: KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: [1u8; 20],
                to: [2u8; 20],
                memo: None,
            }),
            signatures: vec![[99u8; 64]],
        }
    }

    #[test]
    fn test_header_roundtrip() {
        let ts = make_store();
        let thread_id: ThreadId = [1u8; 20];
        let header = sample_header(thread_id);

        ts.save_header(&thread_id, &header).unwrap();
        let loaded = ts.load_header(&thread_id).unwrap();
        assert_eq!(loaded, Some(header));
    }

    #[test]
    fn test_header_not_found() {
        let ts = make_store();
        let thread_id: ThreadId = [99u8; 20];
        assert_eq!(ts.load_header(&thread_id).unwrap(), None);
    }

    #[test]
    fn test_state_roundtrip() {
        let ts = make_store();
        let thread_id: ThreadId = [1u8; 20];
        let state = sample_state();

        ts.save_state(&thread_id, &state).unwrap();
        let loaded = ts.load_state(&thread_id).unwrap();
        assert_eq!(loaded, Some(state));
    }

    #[test]
    fn test_state_not_found() {
        let ts = make_store();
        let thread_id: ThreadId = [99u8; 20];
        assert_eq!(ts.load_state(&thread_id).unwrap(), None);
    }

    #[test]
    fn test_uncommitted_knots_roundtrip() {
        let ts = make_store();
        let thread_id: ThreadId = [1u8; 20];
        let knots = vec![sample_knot()];

        ts.save_uncommitted_knots(&thread_id, &knots).unwrap();
        let loaded = ts.load_uncommitted_knots(&thread_id).unwrap();
        assert_eq!(loaded, knots);
    }

    #[test]
    fn test_uncommitted_knots_empty() {
        let ts = make_store();
        let thread_id: ThreadId = [1u8; 20];
        let loaded = ts.load_uncommitted_knots(&thread_id).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_list_threads() {
        let ts = make_store();

        let id1: ThreadId = [1u8; 20];
        let id2: ThreadId = [2u8; 20];
        let id3: ThreadId = [3u8; 20];

        ts.save_header(&id1, &sample_header(id1)).unwrap();
        ts.save_header(&id2, &sample_header(id2)).unwrap();
        ts.save_header(&id3, &sample_header(id3)).unwrap();

        let mut threads = ts.list_threads().unwrap();
        threads.sort();
        assert_eq!(threads, vec![id1, id2, id3]);
    }

    #[test]
    fn test_list_threads_empty() {
        let ts = make_store();
        let threads = ts.list_threads().unwrap();
        assert!(threads.is_empty());
    }

    #[test]
    fn test_overwrite_header() {
        let ts = make_store();
        let thread_id: ThreadId = [1u8; 20];

        let mut header = sample_header(thread_id);
        ts.save_header(&thread_id, &header).unwrap();

        header.version = 42;
        ts.save_header(&thread_id, &header).unwrap();

        let loaded = ts.load_header(&thread_id).unwrap().unwrap();
        assert_eq!(loaded.version, 42);
    }
}
