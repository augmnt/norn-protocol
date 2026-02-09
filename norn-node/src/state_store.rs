use std::sync::Arc;

use borsh::BorshDeserialize;

use norn_storage::error::StorageError;
use norn_storage::traits::KvStore;
use norn_types::primitives::{Address, LoomId, TokenId};
use norn_types::thread::ThreadState;
use norn_types::weave::WeaveBlock;

use crate::state_manager::{LoomRecord, NameRecord, ThreadMeta, TokenRecord, TransferRecord};

// Key prefixes for each data bucket.
const THREAD_STATE_PREFIX: &[u8] = b"state:thread:";
const THREAD_META_PREFIX: &[u8] = b"state:meta:";
const TRANSFER_PREFIX: &[u8] = b"state:transfer:";
const TRANSFER_COUNT_KEY: &[u8] = b"state:transfer_count";
const NAME_PREFIX: &[u8] = b"state:name:";
const ADDR_NAMES_PREFIX: &[u8] = b"state:addr_names:";
const BLOCK_PREFIX: &[u8] = b"state:block:";
const TOKEN_PREFIX: &[u8] = b"state:token:";
const LOOM_PREFIX: &[u8] = b"state:loom:";
const LOOM_BYTECODE_PREFIX: &[u8] = b"state:loom_bytecode:";
const LOOM_STATE_PREFIX: &[u8] = b"state:loom_state:";
const SCHEMA_VERSION_KEY: &[u8] = b"meta:schema_version";

/// Current schema version. Bump this whenever a breaking change is made to any
/// borsh-serialized type persisted through StateStore.
pub const SCHEMA_VERSION: u32 = 6;

/// Persistent store for StateManager data backed by a KvStore.
pub struct StateStore {
    store: Arc<dyn KvStore>,
}

impl StateStore {
    pub fn new(store: Arc<dyn KvStore>) -> Self {
        Self { store }
    }

    // ── Schema Version ─────────────────────────────────────────────────

    /// Check the persisted schema version against the current binary's version.
    ///
    /// Returns `Ok(())` if compatible, `Err` with a clear message if not.
    /// A store with no schema version key is treated as legacy (version 0).
    pub fn check_schema_version(&self) -> Result<(), StorageError> {
        let stored = match self.store.get(SCHEMA_VERSION_KEY)? {
            Some(bytes) => {
                u32::try_from_slice(&bytes).map_err(|e| StorageError::DeserializationError {
                    reason: format!("failed to read schema version: {}", e),
                })?
            }
            None => 0, // legacy store without version tag
        };

        if stored == SCHEMA_VERSION {
            return Ok(());
        }

        if stored == 0 {
            tracing::warn!(
                "state store has no schema version (legacy data) — \
                 this binary expects schema v{}; run with --reset-state if you see deserialization errors",
                SCHEMA_VERSION
            );
            // Write the current version to upgrade legacy stores.
            self.write_schema_version()?;
            return Ok(());
        }

        Err(StorageError::DeserializationError {
            reason: format!(
                "state store schema version mismatch: store is v{}, binary expects v{} — \
                 run with --reset-state to wipe and restart",
                stored, SCHEMA_VERSION
            ),
        })
    }

    /// Write the current schema version to the store.
    pub fn write_schema_version(&self) -> Result<(), StorageError> {
        let value =
            borsh::to_vec(&SCHEMA_VERSION).map_err(|e| StorageError::SerializationError {
                reason: e.to_string(),
            })?;
        self.store.put(SCHEMA_VERSION_KEY, &value)
    }

    // ── Thread State ────────────────────────────────────────────────────

    pub fn save_thread_state(
        &self,
        address: &Address,
        state: &ThreadState,
    ) -> Result<(), StorageError> {
        let key = self.thread_state_key(address);
        let value = borsh::to_vec(state).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    pub fn load_all_thread_states(&self) -> Result<Vec<(Address, ThreadState)>, StorageError> {
        let pairs = self.store.prefix_scan(THREAD_STATE_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let addr = self.address_from_key(&key, THREAD_STATE_PREFIX.len());
            let state = ThreadState::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push((addr, state));
        }
        Ok(results)
    }

    // ── Thread Meta ─────────────────────────────────────────────────────

    pub fn save_thread_meta(
        &self,
        address: &Address,
        meta: &ThreadMeta,
    ) -> Result<(), StorageError> {
        let key = self.thread_meta_key(address);
        let value = borsh::to_vec(meta).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    pub fn load_all_thread_metas(&self) -> Result<Vec<(Address, ThreadMeta)>, StorageError> {
        let pairs = self.store.prefix_scan(THREAD_META_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let addr = self.address_from_key(&key, THREAD_META_PREFIX.len());
            let meta = ThreadMeta::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push((addr, meta));
        }
        Ok(results)
    }

    // ── Transfers ───────────────────────────────────────────────────────

    pub fn append_transfer(&self, record: &TransferRecord) -> Result<(), StorageError> {
        let seq = self.next_transfer_seq()?;
        let key = self.transfer_key(seq);
        let value = borsh::to_vec(record).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)?;

        // Update counter
        let count_bytes =
            borsh::to_vec(&(seq + 1)).map_err(|e| StorageError::SerializationError {
                reason: e.to_string(),
            })?;
        self.store.put(TRANSFER_COUNT_KEY, &count_bytes)
    }

    pub fn load_all_transfers(&self) -> Result<Vec<TransferRecord>, StorageError> {
        let pairs = self.store.prefix_scan(TRANSFER_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            // Skip the transfer_count key which shares the "state:transfer" prefix
            if key == TRANSFER_COUNT_KEY {
                continue;
            }
            let record = TransferRecord::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push(record);
        }
        Ok(results)
    }

    // ── Names ───────────────────────────────────────────────────────────

    pub fn save_name(&self, name: &str, record: &NameRecord) -> Result<(), StorageError> {
        let key = self.name_key(name);
        let value = borsh::to_vec(record).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    pub fn load_all_names(&self) -> Result<Vec<(String, NameRecord)>, StorageError> {
        let pairs = self.store.prefix_scan(NAME_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let name = String::from_utf8_lossy(&key[NAME_PREFIX.len()..]).to_string();
            let record = NameRecord::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push((name, record));
        }
        Ok(results)
    }

    // ── Address Names ───────────────────────────────────────────────────

    pub fn save_address_names(
        &self,
        address: &Address,
        names: &[String],
    ) -> Result<(), StorageError> {
        let key = self.addr_names_key(address);
        let value = borsh::to_vec(names).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    pub fn load_all_address_names(&self) -> Result<Vec<(Address, Vec<String>)>, StorageError> {
        let pairs = self.store.prefix_scan(ADDR_NAMES_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let addr = self.address_from_key(&key, ADDR_NAMES_PREFIX.len());
            let names = Vec::<String>::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push((addr, names));
        }
        Ok(results)
    }

    // ── Blocks ──────────────────────────────────────────────────────────

    pub fn save_block(&self, block: &WeaveBlock) -> Result<(), StorageError> {
        let key = self.block_key(block.height);
        let value = borsh::to_vec(block).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    pub fn load_all_blocks(&self) -> Result<Vec<WeaveBlock>, StorageError> {
        let pairs = self.store.prefix_scan(BLOCK_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (_key, value) in pairs {
            let block = WeaveBlock::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push(block);
        }
        // Sort by height to ensure ordering
        results.sort_by_key(|b| b.height);
        Ok(results)
    }

    // ── Tokens ──────────────────────────────────────────────────────────

    pub fn save_token(&self, token_id: &TokenId, record: &TokenRecord) -> Result<(), StorageError> {
        let key = self.token_key(token_id);
        let value = borsh::to_vec(record).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    pub fn load_all_tokens(&self) -> Result<Vec<(TokenId, TokenRecord)>, StorageError> {
        let pairs = self.store.prefix_scan(TOKEN_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let token_id = self.token_id_from_key(&key, TOKEN_PREFIX.len());
            let record = TokenRecord::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push((token_id, record));
        }
        Ok(results)
    }

    // ── Looms ───────────────────────────────────────────────────────────

    pub fn save_loom(&self, loom_id: &LoomId, record: &LoomRecord) -> Result<(), StorageError> {
        let key = self.loom_key(loom_id);
        let value = borsh::to_vec(record).map_err(|e| StorageError::SerializationError {
            reason: e.to_string(),
        })?;
        self.store.put(&key, &value)
    }

    pub fn save_loom_bytecode(
        &self,
        loom_id: &LoomId,
        bytecode: &[u8],
    ) -> Result<(), StorageError> {
        let key = self.loom_bytecode_key(loom_id);
        self.store.put(&key, bytecode)
    }

    pub fn save_loom_state(&self, loom_id: &LoomId, state_data: &[u8]) -> Result<(), StorageError> {
        let key = self.loom_state_key(loom_id);
        self.store.put(&key, state_data)
    }

    #[allow(dead_code)]
    pub fn load_loom_bytecode(&self, loom_id: &LoomId) -> Result<Option<Vec<u8>>, StorageError> {
        let key = self.loom_bytecode_key(loom_id);
        self.store.get(&key)
    }

    #[allow(dead_code)]
    pub fn load_loom_state(&self, loom_id: &LoomId) -> Result<Option<Vec<u8>>, StorageError> {
        let key = self.loom_state_key(loom_id);
        self.store.get(&key)
    }

    pub fn load_all_loom_bytecodes(&self) -> Result<Vec<(LoomId, Vec<u8>)>, StorageError> {
        let pairs = self.store.prefix_scan(LOOM_BYTECODE_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let loom_id = self.loom_id_from_key(&key, LOOM_BYTECODE_PREFIX.len());
            results.push((loom_id, value));
        }
        Ok(results)
    }

    pub fn load_all_loom_states(&self) -> Result<Vec<(LoomId, Vec<u8>)>, StorageError> {
        let pairs = self.store.prefix_scan(LOOM_STATE_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let loom_id = self.loom_id_from_key(&key, LOOM_STATE_PREFIX.len());
            results.push((loom_id, value));
        }
        Ok(results)
    }

    pub fn load_all_looms(&self) -> Result<Vec<(LoomId, LoomRecord)>, StorageError> {
        let pairs = self.store.prefix_scan(LOOM_PREFIX)?;
        let mut results = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            let loom_id = self.loom_id_from_key(&key, LOOM_PREFIX.len());
            let record = LoomRecord::try_from_slice(&value).map_err(|e| {
                StorageError::DeserializationError {
                    reason: e.to_string(),
                }
            })?;
            results.push((loom_id, record));
        }
        Ok(results)
    }

    // ── Rebuild ─────────────────────────────────────────────────────────

    /// Rebuild a full StateManager from persisted data.
    pub fn rebuild(&self) -> Result<crate::state_manager::StateManager, StorageError> {
        let thread_states = self.load_all_thread_states()?;
        let thread_metas = self.load_all_thread_metas()?;
        let transfers = self.load_all_transfers()?;
        let names = self.load_all_names()?;
        let address_names = self.load_all_address_names()?;
        let blocks = self.load_all_blocks()?;
        let tokens = self.load_all_tokens()?;
        let looms = self.load_all_looms()?;

        let state_count = thread_states.len();
        let transfer_count = transfers.len();
        let name_count = names.len();
        let block_count = blocks.len();
        let token_count = tokens.len();
        let loom_count = looms.len();

        let mut sm = crate::state_manager::StateManager::from_parts(
            thread_states.into_iter().collect(),
            thread_metas.into_iter().collect(),
            transfers,
            blocks,
            names.into_iter().collect(),
            address_names.into_iter().collect(),
        );

        // Seed token registry from persisted data.
        for (token_id, record) in tokens {
            sm.seed_token(token_id, record);
        }

        // Seed loom registry from persisted data.
        for (loom_id, record) in looms {
            sm.seed_loom(loom_id, record);
        }

        if state_count > 0
            || transfer_count > 0
            || name_count > 0
            || block_count > 0
            || token_count > 0
            || loom_count > 0
        {
            tracing::info!(
                threads = state_count,
                transfers = transfer_count,
                names = name_count,
                blocks = block_count,
                tokens = token_count,
                looms = loom_count,
                "state rebuilt from disk"
            );
        }

        Ok(sm)
    }

    // ── Key helpers ─────────────────────────────────────────────────────

    fn thread_state_key(&self, address: &Address) -> Vec<u8> {
        let mut key = Vec::with_capacity(THREAD_STATE_PREFIX.len() + 20);
        key.extend_from_slice(THREAD_STATE_PREFIX);
        key.extend_from_slice(address);
        key
    }

    fn thread_meta_key(&self, address: &Address) -> Vec<u8> {
        let mut key = Vec::with_capacity(THREAD_META_PREFIX.len() + 20);
        key.extend_from_slice(THREAD_META_PREFIX);
        key.extend_from_slice(address);
        key
    }

    fn transfer_key(&self, seq: u64) -> Vec<u8> {
        let mut key = Vec::with_capacity(TRANSFER_PREFIX.len() + 8);
        key.extend_from_slice(TRANSFER_PREFIX);
        key.extend_from_slice(&seq.to_be_bytes());
        key
    }

    fn name_key(&self, name: &str) -> Vec<u8> {
        let mut key = Vec::with_capacity(NAME_PREFIX.len() + name.len());
        key.extend_from_slice(NAME_PREFIX);
        key.extend_from_slice(name.as_bytes());
        key
    }

    fn addr_names_key(&self, address: &Address) -> Vec<u8> {
        let mut key = Vec::with_capacity(ADDR_NAMES_PREFIX.len() + 20);
        key.extend_from_slice(ADDR_NAMES_PREFIX);
        key.extend_from_slice(address);
        key
    }

    fn block_key(&self, height: u64) -> Vec<u8> {
        let mut key = Vec::with_capacity(BLOCK_PREFIX.len() + 8);
        key.extend_from_slice(BLOCK_PREFIX);
        key.extend_from_slice(&height.to_be_bytes());
        key
    }

    fn token_key(&self, token_id: &TokenId) -> Vec<u8> {
        let mut key = Vec::with_capacity(TOKEN_PREFIX.len() + 32);
        key.extend_from_slice(TOKEN_PREFIX);
        key.extend_from_slice(token_id);
        key
    }

    fn token_id_from_key(&self, key: &[u8], prefix_len: usize) -> TokenId {
        let mut id = [0u8; 32];
        let data = &key[prefix_len..];
        if data.len() >= 32 {
            id.copy_from_slice(&data[..32]);
        }
        id
    }

    fn loom_key(&self, loom_id: &LoomId) -> Vec<u8> {
        let mut key = Vec::with_capacity(LOOM_PREFIX.len() + 32);
        key.extend_from_slice(LOOM_PREFIX);
        key.extend_from_slice(loom_id);
        key
    }

    fn loom_bytecode_key(&self, loom_id: &LoomId) -> Vec<u8> {
        let mut key = Vec::with_capacity(LOOM_BYTECODE_PREFIX.len() + 32);
        key.extend_from_slice(LOOM_BYTECODE_PREFIX);
        key.extend_from_slice(loom_id);
        key
    }

    fn loom_state_key(&self, loom_id: &LoomId) -> Vec<u8> {
        let mut key = Vec::with_capacity(LOOM_STATE_PREFIX.len() + 32);
        key.extend_from_slice(LOOM_STATE_PREFIX);
        key.extend_from_slice(loom_id);
        key
    }

    fn loom_id_from_key(&self, key: &[u8], prefix_len: usize) -> LoomId {
        let mut id = [0u8; 32];
        let data = &key[prefix_len..];
        if data.len() >= 32 {
            id.copy_from_slice(&data[..32]);
        }
        id
    }

    fn address_from_key(&self, key: &[u8], prefix_len: usize) -> Address {
        let mut addr = [0u8; 20];
        let data = &key[prefix_len..];
        if data.len() >= 20 {
            addr.copy_from_slice(&data[..20]);
        }
        addr
    }

    fn next_transfer_seq(&self) -> Result<u64, StorageError> {
        match self.store.get(TRANSFER_COUNT_KEY)? {
            Some(bytes) => {
                u64::try_from_slice(&bytes).map_err(|e| StorageError::DeserializationError {
                    reason: e.to_string(),
                })
            }
            None => Ok(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_storage::memory::MemoryStore;
    use norn_types::primitives::NATIVE_TOKEN_ID;
    use norn_types::weave::WeaveBlock;

    fn make_store() -> StateStore {
        StateStore::new(Arc::new(MemoryStore::new()))
    }

    fn test_address(byte: u8) -> Address {
        [byte; 20]
    }

    #[test]
    fn test_thread_state_roundtrip() {
        let store = make_store();
        let addr = test_address(1);
        let mut state = ThreadState::new();
        state.credit(NATIVE_TOKEN_ID, 1000).unwrap();

        store.save_thread_state(&addr, &state).unwrap();
        let loaded = store.load_all_thread_states().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, addr);
        assert_eq!(loaded[0].1, state);
    }

    #[test]
    fn test_thread_meta_roundtrip() {
        let store = make_store();
        let addr = test_address(2);
        let meta = ThreadMeta {
            owner: [42u8; 32],
            version: 3,
            nonce: 7,
            state_hash: [11u8; 32],
            last_commit_hash: [22u8; 32],
        };

        store.save_thread_meta(&addr, &meta).unwrap();
        let loaded = store.load_all_thread_metas().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, addr);
        assert_eq!(loaded[0].1, meta);
    }

    #[test]
    fn test_transfer_roundtrip() {
        let store = make_store();
        let record = TransferRecord {
            knot_id: [1u8; 32],
            from: test_address(1),
            to: test_address(2),
            token_id: NATIVE_TOKEN_ID,
            amount: 500,
            memo: Some(b"test".to_vec()),
            timestamp: 12345,
            block_height: None,
        };

        store.append_transfer(&record).unwrap();
        store.append_transfer(&record).unwrap();

        let loaded = store.load_all_transfers().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].amount, 500);
    }

    #[test]
    fn test_name_roundtrip() {
        let store = make_store();
        let record = NameRecord {
            owner: test_address(1),
            registered_at: 1000,
            fee_paid: 1_000_000_000_000,
        };

        store.save_name("alice", &record).unwrap();
        let loaded = store.load_all_names().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, "alice");
        assert_eq!(loaded[0].1.owner, test_address(1));
    }

    #[test]
    fn test_address_names_roundtrip() {
        let store = make_store();
        let addr = test_address(3);
        let names = vec!["alice".to_string(), "bob".to_string()];

        store.save_address_names(&addr, &names).unwrap();
        let loaded = store.load_all_address_names().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1, names);
    }

    #[test]
    fn test_block_roundtrip() {
        let store = make_store();
        let block = WeaveBlock {
            height: 1,
            hash: [1u8; 32],
            prev_hash: [0u8; 32],
            commitments_root: [0u8; 32],
            registrations_root: [0u8; 32],
            anchors_root: [0u8; 32],
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
            timestamp: 1000,
            proposer: [0u8; 32],
            validator_signatures: vec![],
        };

        store.save_block(&block).unwrap();
        let loaded = store.load_all_blocks().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].height, 1);
    }

    #[test]
    fn test_rebuild_empty() {
        let store = make_store();
        let sm = store.rebuild().unwrap();
        assert_eq!(sm.latest_block_height(), 0);
    }

    #[test]
    fn test_schema_version_fresh_store() {
        let store = make_store();
        // Fresh store has no version key — check should succeed (legacy path).
        assert!(store.check_schema_version().is_ok());
    }

    #[test]
    fn test_schema_version_write_and_check() {
        let store = make_store();
        store.write_schema_version().unwrap();
        assert!(store.check_schema_version().is_ok());
    }

    #[test]
    fn test_schema_version_mismatch() {
        let store = make_store();
        // Write a future version that doesn't match SCHEMA_VERSION.
        let future_version: u32 = SCHEMA_VERSION + 10;
        let value = borsh::to_vec(&future_version).unwrap();
        store.store.put(SCHEMA_VERSION_KEY, &value).unwrap();

        let result = store.check_schema_version();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("schema version mismatch"));
        assert!(err_msg.contains("--reset-state"));
    }
}
