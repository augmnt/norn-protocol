use std::collections::{HashMap, HashSet};

use borsh::{BorshDeserialize, BorshSerialize};

use norn_crypto::merkle::SparseMerkleTree;
use norn_types::constants::MAX_SUPPLY;
use norn_types::error::NornError;
use norn_types::loom::LOOM_DEPLOY_FEE;
use norn_types::name::NAME_REGISTRATION_FEE;
use norn_types::primitives::{Address, Amount, Hash, LoomId, PublicKey, TokenId, NATIVE_TOKEN_ID};
use norn_types::thread::ThreadState;
use norn_types::token::TOKEN_CREATION_FEE;
use norn_types::weave::WeaveBlock;

// Re-export for backward compatibility (used by wallet CLI and state_store).
pub use norn_types::name::validate_name;

/// A record of a registered name.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct NameRecord {
    pub owner: Address,
    pub registered_at: u64,
    pub fee_paid: Amount,
}

/// A record of a registered token (NT-1).
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TokenRecord {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub max_supply: Amount,
    pub current_supply: Amount,
    pub creator: Address,
    pub created_at: u64,
}

/// A record of a deployed loom (smart contract).
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LoomRecord {
    pub name: String,
    pub operator: PublicKey,
    pub max_participants: usize,
    pub min_participants: usize,
    pub active: bool,
    pub deployed_at: u64,
}

/// Metadata tracked per thread beyond its ThreadState.
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
#[allow(dead_code)] // Fields accessed via borsh serialization and pattern matching; nonce reserved for future use
pub struct ThreadMeta {
    pub owner: PublicKey,
    pub version: u64,
    pub nonce: u64,
    pub state_hash: Hash,
    pub last_commit_hash: Hash,
}

/// A record of a token transfer (for history queries).
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TransferRecord {
    pub knot_id: Hash,
    pub from: Address,
    pub to: Address,
    pub token_id: TokenId,
    pub amount: Amount,
    pub memo: Option<Vec<u8>>,
    pub timestamp: u64,
    pub block_height: Option<u64>,
}

/// Maximum number of blocks kept in memory (older blocks available via SQLite).
const MAX_BLOCK_ARCHIVE: usize = 1000;
/// Maximum number of transfer records kept in memory.
const MAX_TRANSFER_LOG: usize = 10_000;
/// Maximum number of knot IDs tracked for dedup.
const MAX_KNOWN_KNOT_IDS: usize = 50_000;

/// Node-side state manager that tracks balances, history, and blocks
/// alongside the WeaveEngine's consensus-level tracking.
pub struct StateManager {
    thread_states: HashMap<Address, ThreadState>,
    thread_meta: HashMap<Address, ThreadMeta>,
    transfer_log: Vec<TransferRecord>,
    block_archive: Vec<WeaveBlock>,
    name_registry: HashMap<String, NameRecord>,
    address_names: HashMap<Address, Vec<String>>,
    state_store: Option<crate::state_store::StateStore>,
    /// Known knot IDs for transfer dedup (prevents double-applying from gossip + block).
    known_knot_ids: HashSet<Hash>,
    /// Cached total supply of native tokens (updated on credit/debit).
    total_supply_cache: Amount,
    /// Registry of NT-1 tokens by token_id.
    token_registry: HashMap<TokenId, TokenRecord>,
    /// Index from symbol to token_id for symbol-based lookups.
    symbol_index: HashMap<String, TokenId>,
    /// Registry of deployed looms by loom_id.
    loom_registry: HashMap<LoomId, LoomRecord>,
    /// Sparse Merkle tree for computing cumulative state roots.
    state_smt: SparseMerkleTree,
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            thread_states: HashMap::new(),
            thread_meta: HashMap::new(),
            transfer_log: Vec::new(),
            block_archive: Vec::new(),
            name_registry: HashMap::new(),
            address_names: HashMap::new(),
            state_store: None,
            known_knot_ids: HashSet::new(),
            total_supply_cache: 0,
            token_registry: HashMap::new(),
            symbol_index: HashMap::new(),
            loom_registry: HashMap::new(),
            state_smt: SparseMerkleTree::new(),
        }
    }

    /// Reconstruct a StateManager from pre-loaded data (used by state_store::rebuild).
    pub fn from_parts(
        thread_states: HashMap<Address, ThreadState>,
        thread_meta: HashMap<Address, ThreadMeta>,
        transfer_log: Vec<TransferRecord>,
        block_archive: Vec<WeaveBlock>,
        name_registry: HashMap<String, NameRecord>,
        address_names: HashMap<Address, Vec<String>>,
    ) -> Self {
        // Compute total supply from loaded state.
        let total_supply_cache = thread_states
            .values()
            .map(|s| s.balance(&NATIVE_TOKEN_ID))
            .sum();
        let known_knot_ids = transfer_log.iter().map(|r| r.knot_id).collect();

        // Rebuild the SMT from all persisted balances.
        let mut state_smt = SparseMerkleTree::new();
        for (address, state) in &thread_states {
            for (token_id, &balance) in &state.balances {
                if balance > 0 {
                    let mut data = Vec::with_capacity(20 + 32);
                    data.extend_from_slice(address);
                    data.extend_from_slice(token_id);
                    let key = norn_crypto::hash::blake3_hash(&data);
                    state_smt.insert(key, balance.to_le_bytes().to_vec());
                }
            }
        }

        Self {
            thread_states,
            thread_meta,
            transfer_log,
            block_archive,
            name_registry,
            address_names,
            state_store: None,
            known_knot_ids,
            total_supply_cache,
            token_registry: HashMap::new(),
            symbol_index: HashMap::new(),
            loom_registry: HashMap::new(),
            state_smt,
        }
    }

    /// Attach a state store for write-through persistence.
    pub fn set_store(&mut self, store: crate::state_store::StateStore) {
        self.state_store = Some(store);
    }

    /// Get a reference to the underlying state store (if attached).
    pub fn store(&self) -> Option<&crate::state_store::StateStore> {
        self.state_store.as_ref()
    }

    /// Register a thread with its owner public key.
    pub fn register_thread(&mut self, address: Address, pubkey: PublicKey) {
        if self.thread_states.contains_key(&address) {
            return;
        }
        let state = ThreadState::new();
        let state_hash = norn_thread::state::compute_state_hash(&state);
        let meta = ThreadMeta {
            owner: pubkey,
            version: 0,
            nonce: 0,
            state_hash,
            last_commit_hash: [0u8; 32],
        };
        self.thread_states.insert(address, state.clone());
        self.thread_meta.insert(address, meta.clone());

        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_thread_state(&address, &state) {
                tracing::warn!(
                    "Failed to persist thread state for {}: {}",
                    hex::encode(address),
                    e
                );
            }
            if let Err(e) = store.save_thread_meta(&address, &meta) {
                tracing::warn!(
                    "Failed to persist thread meta for {}: {}",
                    hex::encode(address),
                    e
                );
            }
        }
    }

    /// Check if an address is registered.
    pub fn is_registered(&self, address: &Address) -> bool {
        self.thread_states.contains_key(address)
    }

    /// Auto-register a thread if not already present.
    /// Uses the provided pubkey when available, otherwise falls back to a zero
    /// pubkey (e.g., for transfer recipients whose key is unknown).
    pub fn auto_register_if_needed(&mut self, address: Address) {
        if !self.is_registered(&address) {
            self.register_thread(address, [0u8; 32]);
        }
    }

    /// Auto-register a thread with a known public key if not already present.
    pub fn auto_register_with_pubkey(&mut self, address: Address, pubkey: PublicKey) {
        if !self.is_registered(&address) {
            self.register_thread(address, pubkey);
        }
    }

    /// Check if a transfer with the given knot_id has already been applied.
    pub fn has_transfer(&self, knot_id: &Hash) -> bool {
        self.known_knot_ids.contains(knot_id)
    }

    /// Get the total circulating supply of native tokens.
    #[allow(dead_code)] // Used in tests and as a pub API
    pub fn total_supply(&self) -> Amount {
        self.total_supply_cache
    }

    /// Compute the current state root from the sparse Merkle tree.
    pub fn state_root(&mut self) -> Hash {
        self.state_smt.root()
    }

    /// Generate a Merkle proof for a specific balance.
    pub fn state_proof(
        &mut self,
        address: &Address,
        token_id: &TokenId,
    ) -> norn_crypto::merkle::MerkleProof {
        let smt_key = self.smt_key(address, token_id);
        self.state_smt.prove(&smt_key)
    }

    /// Compute the SMT key for a balance entry: BLAKE3(address ++ token_id).
    fn smt_key(&self, address: &Address, token_id: &TokenId) -> Hash {
        let mut data = Vec::with_capacity(20 + 32);
        data.extend_from_slice(address);
        data.extend_from_slice(token_id);
        norn_crypto::hash::blake3_hash(&data)
    }

    /// Update the SMT for a balance change.
    fn update_smt(&mut self, address: &Address, token_id: &TokenId) {
        let balance = self
            .thread_states
            .get(address)
            .map(|s| s.balance(token_id))
            .unwrap_or(0);
        let key = self.smt_key(address, token_id);
        self.state_smt.insert(key, balance.to_le_bytes().to_vec());
    }

    /// Credit tokens to an address (e.g., faucet).
    /// For native tokens, enforces MAX_SUPPLY cap.
    pub fn credit(
        &mut self,
        address: Address,
        token_id: TokenId,
        amount: Amount,
    ) -> Result<(), NornError> {
        // Enforce MAX_SUPPLY for native token.
        if token_id == NATIVE_TOKEN_ID {
            let new_total = self
                .total_supply_cache
                .checked_add(amount)
                .ok_or(NornError::InvalidAmount)?;
            if new_total > MAX_SUPPLY {
                return Err(NornError::SupplyCapExceeded {
                    current: self.total_supply_cache,
                    requested: amount,
                    max: MAX_SUPPLY,
                });
            }
        }

        let state = self
            .thread_states
            .get_mut(&address)
            .ok_or(NornError::ThreadNotFound(address))?;
        state.credit(token_id, amount)?;

        // Update cached total supply for native token.
        if token_id == NATIVE_TOKEN_ID {
            self.total_supply_cache += amount;
        }

        // Update state hash in meta
        if let Some(meta) = self.thread_meta.get_mut(&address) {
            meta.state_hash = norn_thread::state::compute_state_hash(state);
        }

        // Update SMT.
        self.update_smt(&address, &token_id);

        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) =
                store.save_thread_state(&address, self.thread_states.get(&address).unwrap())
            {
                tracing::warn!("Failed to persist thread state: {}", e);
            }
            if let Some(meta) = self.thread_meta.get(&address) {
                if let Err(e) = store.save_thread_meta(&address, meta) {
                    tracing::warn!("Failed to persist thread meta: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Apply a transfer: debit sender, credit receiver, log it.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: TokenId,
        amount: Amount,
        knot_id: Hash,
        memo: Option<Vec<u8>>,
        timestamp: u64,
    ) -> Result<(), NornError> {
        if amount == 0 {
            return Err(NornError::InvalidAmount);
        }

        // Check sender balance
        let sender_state = self
            .thread_states
            .get(&from)
            .ok_or(NornError::ThreadNotFound(from))?;
        if !sender_state.has_balance(&token_id, amount) {
            return Err(NornError::InsufficientBalance {
                available: sender_state.balance(&token_id),
                required: amount,
            });
        }

        // Debit sender
        let sender_state = self.thread_states.get_mut(&from).unwrap();
        sender_state.debit(&token_id, amount);

        // Credit receiver
        let receiver_state = self
            .thread_states
            .get_mut(&to)
            .ok_or(NornError::ThreadNotFound(to))?;
        receiver_state.credit(token_id, amount)?;

        // Update state hashes
        if let Some(meta) = self.thread_meta.get_mut(&from) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&from).unwrap());
        }
        if let Some(meta) = self.thread_meta.get_mut(&to) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&to).unwrap());
        }

        // Update SMT for both sender and receiver.
        self.update_smt(&from, &token_id);
        self.update_smt(&to, &token_id);

        // Track knot_id for dedup.
        self.known_knot_ids.insert(knot_id);

        // Log the transfer
        let record = TransferRecord {
            knot_id,
            from,
            to,
            token_id,
            amount,
            memo,
            timestamp,
            block_height: None,
        };
        self.transfer_log.push(record.clone());

        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_thread_state(&from, self.thread_states.get(&from).unwrap()) {
                tracing::warn!("Failed to persist sender state: {}", e);
            }
            if let Err(e) = store.save_thread_state(&to, self.thread_states.get(&to).unwrap()) {
                tracing::warn!("Failed to persist receiver state: {}", e);
            }
            if let Some(meta) = self.thread_meta.get(&from) {
                if let Err(e) = store.save_thread_meta(&from, meta) {
                    tracing::warn!("Failed to persist sender meta: {}", e);
                }
            }
            if let Some(meta) = self.thread_meta.get(&to) {
                if let Err(e) = store.save_thread_meta(&to, meta) {
                    tracing::warn!("Failed to persist receiver meta: {}", e);
                }
            }
            if let Err(e) = store.append_transfer(&record) {
                tracing::warn!("Failed to persist transfer record: {}", e);
            }
        }

        Ok(())
    }

    /// Apply a transfer received from a peer block or P2P gossip.
    /// Debits the sender (best-effort — warns on insufficient balance) and
    /// credits the recipient so that balances converge across nodes.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_peer_transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: TokenId,
        amount: Amount,
        knot_id: Hash,
        memo: Option<Vec<u8>>,
        timestamp: u64,
    ) -> Result<(), NornError> {
        if amount == 0 {
            return Err(NornError::InvalidAmount);
        }

        // Debit sender (best-effort: warn if insufficient balance).
        if let Some(sender_state) = self.thread_states.get(&from) {
            if sender_state.has_balance(&token_id, amount) {
                let sender_state = self.thread_states.get_mut(&from).unwrap();
                sender_state.debit(&token_id, amount);

                // Update sender state hash.
                if let Some(meta) = self.thread_meta.get_mut(&from) {
                    meta.state_hash = norn_thread::state::compute_state_hash(
                        self.thread_states.get(&from).unwrap(),
                    );
                }

                // Update SMT for sender.
                self.update_smt(&from, &token_id);
            } else {
                tracing::warn!(
                    "peer transfer: sender {} has insufficient balance for {} (available: {})",
                    hex::encode(from),
                    amount,
                    sender_state.balance(&token_id),
                );
            }
        } else {
            tracing::warn!(
                "peer transfer: sender {} not registered, skipping debit",
                hex::encode(from),
            );
        }

        // Credit receiver.
        let receiver_state = self
            .thread_states
            .get_mut(&to)
            .ok_or(NornError::ThreadNotFound(to))?;
        receiver_state.credit(token_id, amount)?;

        // Update receiver state hash.
        if let Some(meta) = self.thread_meta.get_mut(&to) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&to).unwrap());
        }

        // Update SMT for receiver.
        self.update_smt(&to, &token_id);

        // Track knot_id for dedup.
        self.known_knot_ids.insert(knot_id);

        // Log the transfer.
        let record = TransferRecord {
            knot_id,
            from,
            to,
            token_id,
            amount,
            memo,
            timestamp,
            block_height: None,
        };
        self.transfer_log.push(record.clone());

        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_thread_state(&from, self.thread_states.get(&from).unwrap()) {
                tracing::warn!("Failed to persist sender state: {}", e);
            }
            if let Some(meta) = self.thread_meta.get(&from) {
                if let Err(e) = store.save_thread_meta(&from, meta) {
                    tracing::warn!("Failed to persist sender meta: {}", e);
                }
            }
            if let Err(e) = store.save_thread_state(&to, self.thread_states.get(&to).unwrap()) {
                tracing::warn!("Failed to persist receiver state: {}", e);
            }
            if let Some(meta) = self.thread_meta.get(&to) {
                if let Err(e) = store.save_thread_meta(&to, meta) {
                    tracing::warn!("Failed to persist receiver meta: {}", e);
                }
            }
            if let Err(e) = store.append_transfer(&record) {
                tracing::warn!("Failed to persist transfer record: {}", e);
            }
        }

        Ok(())
    }

    /// Get balance for an address and token.
    pub fn get_balance(&self, address: &Address, token_id: &TokenId) -> Amount {
        self.thread_states
            .get(address)
            .map(|s| s.balance(token_id))
            .unwrap_or(0)
    }

    /// Debit a commitment fee from an address. Logs a warning if the address
    /// has insufficient balance (does not fail the block).
    pub fn debit_fee(&mut self, address: Address, fee: Amount) {
        if fee == 0 {
            return;
        }
        let state = match self.thread_states.get_mut(&address) {
            Some(s) => s,
            None => {
                tracing::warn!(
                    "fee debit: address {} not registered, skipping",
                    hex::encode(address)
                );
                return;
            }
        };
        if !state.has_balance(&NATIVE_TOKEN_ID, fee) {
            tracing::warn!(
                "fee debit: address {} has insufficient balance for fee {}",
                hex::encode(address),
                fee
            );
            return;
        }
        state.debit(&NATIVE_TOKEN_ID, fee);

        // Fee is burned (not credited to anyone), so decrement total supply.
        self.total_supply_cache = self.total_supply_cache.saturating_sub(fee);

        // Update state hash in meta.
        if let Some(meta) = self.thread_meta.get_mut(&address) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&address).unwrap());
        }

        // Update SMT.
        self.update_smt(&address, &NATIVE_TOKEN_ID);

        // Persist.
        if let Some(ref store) = self.state_store {
            if let Err(e) =
                store.save_thread_state(&address, self.thread_states.get(&address).unwrap())
            {
                tracing::warn!("Failed to persist thread state after fee debit: {}", e);
            }
            if let Some(meta) = self.thread_meta.get(&address) {
                if let Err(e) = store.save_thread_meta(&address, meta) {
                    tracing::warn!("Failed to persist thread meta after fee debit: {}", e);
                }
            }
        }
    }

    /// Get a reference to a thread's state.
    pub fn get_thread_state(&self, address: &Address) -> Option<&ThreadState> {
        self.thread_states.get(address)
    }

    /// Get a reference to a thread's metadata.
    pub fn get_thread_meta(&self, address: &Address) -> Option<&ThreadMeta> {
        self.thread_meta.get(address)
    }

    /// Get transfer history for an address (sent or received), with limit and offset.
    pub fn get_history(
        &self,
        address: &Address,
        limit: usize,
        offset: usize,
    ) -> Vec<&TransferRecord> {
        self.transfer_log
            .iter()
            .rev()
            .filter(|r| r.from == *address || r.to == *address)
            .skip(offset)
            .take(limit)
            .collect()
    }

    /// Record a commitment update for a thread.
    pub fn record_commitment(
        &mut self,
        address: Address,
        version: u64,
        state_hash: Hash,
        prev_hash: Hash,
        _knot_count: u64,
    ) {
        if let Some(meta) = self.thread_meta.get_mut(&address) {
            meta.version = version;
            meta.state_hash = state_hash;
            meta.last_commit_hash = prev_hash;
        }
    }

    /// Archive a produced block. Evicts oldest blocks from memory when the
    /// archive exceeds `MAX_BLOCK_ARCHIVE` (older blocks remain in SQLite).
    pub fn archive_block(&mut self, block: WeaveBlock) {
        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_block(&block) {
                tracing::warn!("Failed to persist block {}: {}", block.height, e);
            }
        }
        self.block_archive.push(block);

        // Evict oldest blocks from memory (they're persisted to disk).
        if self.block_archive.len() > MAX_BLOCK_ARCHIVE {
            let excess = self.block_archive.len() - MAX_BLOCK_ARCHIVE;
            self.block_archive.drain(..excess);
        }

        // Evict oldest transfer records from memory.
        if self.transfer_log.len() > MAX_TRANSFER_LOG {
            let excess = self.transfer_log.len() - MAX_TRANSFER_LOG;
            self.transfer_log.drain(..excess);
        }

        // Prune oldest knot IDs when the set grows too large.
        if self.known_knot_ids.len() > MAX_KNOWN_KNOT_IDS {
            // HashSet has no ordering, so we clear half and rebuild from recent transfers.
            self.known_knot_ids.clear();
            for record in &self.transfer_log {
                self.known_knot_ids.insert(record.knot_id);
            }
        }
    }

    /// Get a block by height.
    pub fn get_block(&self, height: u64) -> Option<&WeaveBlock> {
        self.block_archive.iter().find(|b| b.height == height)
    }

    /// Get the latest block height.
    pub fn latest_block_height(&self) -> u64 {
        self.block_archive.last().map(|b| b.height).unwrap_or(0)
    }

    /// Log a faucet credit as a transfer record (from zero-address).
    pub fn log_faucet_credit(&mut self, address: Address, amount: Amount, timestamp: u64) {
        let record = TransferRecord {
            knot_id: [0u8; 32],
            from: [0u8; 20], // zero address = faucet
            to: address,
            token_id: NATIVE_TOKEN_ID,
            amount,
            memo: Some(b"faucet".to_vec()),
            timestamp,
            block_height: None,
        };
        self.transfer_log.push(record.clone());

        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_thread_state(
                &address,
                self.thread_states
                    .get(&address)
                    .unwrap_or(&ThreadState::new()),
            ) {
                tracing::warn!("Failed to persist thread state after faucet: {}", e);
            }
            if let Err(e) = store.append_transfer(&record) {
                tracing::warn!("Failed to persist faucet transfer: {}", e);
            }
        }
    }

    /// Register a name for an address. Validates the name, checks uniqueness,
    /// deducts the registration fee (burned), and records the name.
    /// Used for local name registrations where the fee should be deducted.
    pub fn register_name(
        &mut self,
        name: &str,
        owner: Address,
        timestamp: u64,
    ) -> Result<(), NornError> {
        validate_name(name)?;

        if self.name_registry.contains_key(name) {
            return Err(NornError::NameAlreadyRegistered(name.to_string()));
        }

        // Debit the registration fee (burn it).
        let sender_state = self
            .thread_states
            .get(&owner)
            .ok_or(NornError::ThreadNotFound(owner))?;
        if !sender_state.has_balance(&NATIVE_TOKEN_ID, NAME_REGISTRATION_FEE) {
            return Err(NornError::InsufficientBalance {
                available: sender_state.balance(&NATIVE_TOKEN_ID),
                required: NAME_REGISTRATION_FEE,
            });
        }

        let sender_state = self.thread_states.get_mut(&owner).unwrap();
        sender_state.debit(&NATIVE_TOKEN_ID, NAME_REGISTRATION_FEE);

        // Registration fee is burned, so decrement total supply.
        self.total_supply_cache = self
            .total_supply_cache
            .saturating_sub(NAME_REGISTRATION_FEE);

        // Update state hash.
        if let Some(meta) = self.thread_meta.get_mut(&owner) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&owner).unwrap());
        }

        // Update SMT.
        self.update_smt(&owner, &NATIVE_TOKEN_ID);

        // Log the fee burn as a transfer to the zero address.
        let fee_record = TransferRecord {
            knot_id: [0u8; 32],
            from: owner,
            to: [0u8; 20], // burn address
            token_id: NATIVE_TOKEN_ID,
            amount: NAME_REGISTRATION_FEE,
            memo: Some(format!("name registration: {}", name).into_bytes()),
            timestamp,
            block_height: None,
        };
        self.transfer_log.push(fee_record.clone());

        // Record the name.
        let name_record = NameRecord {
            owner,
            registered_at: timestamp,
            fee_paid: NAME_REGISTRATION_FEE,
        };
        self.name_registry
            .insert(name.to_string(), name_record.clone());
        self.address_names
            .entry(owner)
            .or_default()
            .push(name.to_string());

        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_thread_state(&owner, self.thread_states.get(&owner).unwrap())
            {
                tracing::warn!(
                    "Failed to persist thread state after name registration: {}",
                    e
                );
            }
            if let Some(meta) = self.thread_meta.get(&owner) {
                if let Err(e) = store.save_thread_meta(&owner, meta) {
                    tracing::warn!(
                        "Failed to persist thread meta after name registration: {}",
                        e
                    );
                }
            }
            if let Err(e) = store.save_name(name, &name_record) {
                tracing::warn!("Failed to persist name record: {}", e);
            }
            if let Some(names) = self.address_names.get(&owner) {
                if let Err(e) = store.save_address_names(&owner, names) {
                    tracing::warn!("Failed to persist address names: {}", e);
                }
            }
            if let Err(e) = store.append_transfer(&fee_record) {
                tracing::warn!("Failed to persist name registration fee transfer: {}", e);
            }
        }

        Ok(())
    }

    /// Apply a name registration received from a peer block.
    /// Unlike `register_name()`, this skips fee deduction (already burned on the
    /// originating node) and auto-registers the owner thread if needed.
    pub fn apply_peer_name_registration(
        &mut self,
        name: &str,
        owner: Address,
        owner_pubkey: PublicKey,
        timestamp: u64,
        fee_paid: Amount,
    ) -> Result<(), NornError> {
        validate_name(name)?;

        if self.name_registry.contains_key(name) {
            return Err(NornError::NameAlreadyRegistered(name.to_string()));
        }

        // Auto-register the owner thread with their real pubkey.
        if !self.is_registered(&owner) {
            self.register_thread(owner, owner_pubkey);
        }

        // Record the name (no fee deduction — already burned on originating node).
        let name_record = NameRecord {
            owner,
            registered_at: timestamp,
            fee_paid,
        };
        self.name_registry
            .insert(name.to_string(), name_record.clone());
        self.address_names
            .entry(owner)
            .or_default()
            .push(name.to_string());

        // Persist
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_name(name, &name_record) {
                tracing::warn!("Failed to persist name record: {}", e);
            }
            if let Some(names) = self.address_names.get(&owner) {
                if let Err(e) = store.save_address_names(&owner, names) {
                    tracing::warn!("Failed to persist address names: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Resolve a name to its record.
    pub fn resolve_name(&self, name: &str) -> Option<&NameRecord> {
        self.name_registry.get(name)
    }

    /// Iterate over all registered name strings.
    pub fn registered_names(&self) -> impl Iterator<Item = &str> {
        self.name_registry.keys().map(|s| s.as_str())
    }

    /// Iterate over all registered thread IDs (addresses).
    pub fn registered_thread_ids(&self) -> impl Iterator<Item = &Address> {
        self.thread_states.keys()
    }

    /// Get all names owned by an address.
    pub fn names_for_address(&self, address: &Address) -> Vec<&str> {
        self.address_names
            .get(address)
            .map(|names| names.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    // ─── Token Operations (NT-1) ─────────────────────────────────────────────

    /// Create a new token (solo path — deducts creation fee from creator).
    /// Returns the computed token_id.
    #[allow(clippy::too_many_arguments)]
    pub fn create_token(
        &mut self,
        name: &str,
        symbol: &str,
        decimals: u8,
        max_supply: Amount,
        initial_supply: Amount,
        creator: Address,
        timestamp: u64,
    ) -> Result<TokenId, NornError> {
        let token_id = norn_types::token::compute_token_id(
            &creator, name, symbol, decimals, max_supply, timestamp,
        );

        if self.token_registry.contains_key(&token_id) {
            return Err(NornError::TokenAlreadyExists(hex::encode(token_id)));
        }
        if self.symbol_index.contains_key(symbol) {
            return Err(NornError::TokenSymbolTaken(symbol.to_string()));
        }

        // Deduct creation fee.
        let sender_state = self
            .thread_states
            .get(&creator)
            .ok_or(NornError::ThreadNotFound(creator))?;
        if !sender_state.has_balance(&NATIVE_TOKEN_ID, TOKEN_CREATION_FEE) {
            return Err(NornError::InsufficientBalance {
                available: sender_state.balance(&NATIVE_TOKEN_ID),
                required: TOKEN_CREATION_FEE,
            });
        }

        let sender_state = self.thread_states.get_mut(&creator).unwrap();
        sender_state.debit(&NATIVE_TOKEN_ID, TOKEN_CREATION_FEE);
        self.total_supply_cache = self.total_supply_cache.saturating_sub(TOKEN_CREATION_FEE);

        // Mint initial supply to creator.
        if initial_supply > 0 {
            let sender_state = self.thread_states.get_mut(&creator).unwrap();
            sender_state.credit(token_id, initial_supply)?;
        }

        // Update state hash.
        if let Some(meta) = self.thread_meta.get_mut(&creator) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&creator).unwrap());
        }

        // Update SMT (native fee debit + optional token credit).
        self.update_smt(&creator, &NATIVE_TOKEN_ID);
        if initial_supply > 0 {
            self.update_smt(&creator, &token_id);
        }

        // Register the token.
        let record = TokenRecord {
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimals,
            max_supply,
            current_supply: initial_supply,
            creator,
            created_at: timestamp,
        };
        self.token_registry.insert(token_id, record.clone());
        self.symbol_index.insert(symbol.to_string(), token_id);

        // Persist.
        if let Some(ref store) = self.state_store {
            if let Err(e) =
                store.save_thread_state(&creator, self.thread_states.get(&creator).unwrap())
            {
                tracing::warn!(
                    "Failed to persist creator state after token creation: {}",
                    e
                );
            }
            if let Some(meta) = self.thread_meta.get(&creator) {
                if let Err(e) = store.save_thread_meta(&creator, meta) {
                    tracing::warn!("Failed to persist creator meta after token creation: {}", e);
                }
            }
            if let Err(e) = store.save_token(&token_id, &record) {
                tracing::warn!("Failed to persist token record: {}", e);
            }
        }

        Ok(token_id)
    }

    /// Apply a token creation from a peer block (skips fee deduction).
    #[allow(clippy::too_many_arguments)]
    pub fn apply_peer_token_creation(
        &mut self,
        name: &str,
        symbol: &str,
        decimals: u8,
        max_supply: Amount,
        initial_supply: Amount,
        creator: Address,
        creator_pubkey: PublicKey,
        timestamp: u64,
    ) -> Result<TokenId, NornError> {
        let token_id = norn_types::token::compute_token_id(
            &creator, name, symbol, decimals, max_supply, timestamp,
        );

        if self.token_registry.contains_key(&token_id) {
            return Err(NornError::TokenAlreadyExists(hex::encode(token_id)));
        }

        // Auto-register creator with real pubkey.
        self.auto_register_with_pubkey(creator, creator_pubkey);

        // Mint initial supply to creator.
        if initial_supply > 0 {
            let state = self
                .thread_states
                .get_mut(&creator)
                .ok_or(NornError::ThreadNotFound(creator))?;
            state.credit(token_id, initial_supply)?;

            // Update state hash.
            if let Some(meta) = self.thread_meta.get_mut(&creator) {
                meta.state_hash = norn_thread::state::compute_state_hash(
                    self.thread_states.get(&creator).unwrap(),
                );
            }

            // Update SMT for initial supply credit.
            self.update_smt(&creator, &token_id);
        }

        let record = TokenRecord {
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimals,
            max_supply,
            current_supply: initial_supply,
            creator,
            created_at: timestamp,
        };
        self.token_registry.insert(token_id, record.clone());
        self.symbol_index.insert(symbol.to_string(), token_id);

        // Persist.
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_token(&token_id, &record) {
                tracing::warn!("Failed to persist token record: {}", e);
            }
            if let Err(e) =
                store.save_thread_state(&creator, self.thread_states.get(&creator).unwrap())
            {
                tracing::warn!(
                    "Failed to persist creator state after peer token creation: {}",
                    e
                );
            }
        }

        Ok(token_id)
    }

    /// Mint tokens (solo path — credits to recipient, updates supply).
    pub fn mint_token(
        &mut self,
        token_id: TokenId,
        to: Address,
        amount: Amount,
    ) -> Result<(), NornError> {
        let record = self
            .token_registry
            .get(&token_id)
            .ok_or_else(|| NornError::TokenNotFound(hex::encode(token_id)))?;

        // Check supply cap.
        if record.max_supply > 0 {
            let new_supply = record.current_supply.saturating_add(amount);
            if new_supply > record.max_supply {
                return Err(NornError::TokenSupplyCapExceeded {
                    current: record.current_supply,
                    requested: amount,
                    max: record.max_supply,
                });
            }
        }

        // Credit recipient.
        self.auto_register_if_needed(to);
        let state = self
            .thread_states
            .get_mut(&to)
            .ok_or(NornError::ThreadNotFound(to))?;
        state.credit(token_id, amount)?;

        // Update state hash.
        if let Some(meta) = self.thread_meta.get_mut(&to) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&to).unwrap());
        }

        // Update SMT.
        self.update_smt(&to, &token_id);

        // Update supply.
        let record = self.token_registry.get_mut(&token_id).unwrap();
        record.current_supply = record.current_supply.saturating_add(amount);

        // Persist.
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_thread_state(&to, self.thread_states.get(&to).unwrap()) {
                tracing::warn!("Failed to persist recipient state after mint: {}", e);
            }
            if let Err(e) = store.save_token(&token_id, self.token_registry.get(&token_id).unwrap())
            {
                tracing::warn!("Failed to persist token record after mint: {}", e);
            }
        }

        Ok(())
    }

    /// Apply a token mint from a peer block (same logic, just different call context).
    pub fn apply_peer_token_mint(
        &mut self,
        token_id: TokenId,
        to: Address,
        amount: Amount,
    ) -> Result<(), NornError> {
        self.mint_token(token_id, to, amount)
    }

    /// Burn tokens (solo path — debits from burner, updates supply).
    pub fn burn_token(
        &mut self,
        token_id: TokenId,
        burner: Address,
        amount: Amount,
    ) -> Result<(), NornError> {
        if !self.token_registry.contains_key(&token_id) {
            return Err(NornError::TokenNotFound(hex::encode(token_id)));
        }

        // Check balance.
        let state = self
            .thread_states
            .get(&burner)
            .ok_or(NornError::ThreadNotFound(burner))?;
        if !state.has_balance(&token_id, amount) {
            return Err(NornError::InsufficientBalance {
                available: state.balance(&token_id),
                required: amount,
            });
        }

        // Debit burner.
        let state = self.thread_states.get_mut(&burner).unwrap();
        state.debit(&token_id, amount);

        // Update state hash.
        if let Some(meta) = self.thread_meta.get_mut(&burner) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&burner).unwrap());
        }

        // Update SMT.
        self.update_smt(&burner, &token_id);

        // Update supply.
        let record = self.token_registry.get_mut(&token_id).unwrap();
        record.current_supply = record.current_supply.saturating_sub(amount);

        // Persist.
        if let Some(ref store) = self.state_store {
            if let Err(e) =
                store.save_thread_state(&burner, self.thread_states.get(&burner).unwrap())
            {
                tracing::warn!("Failed to persist burner state after burn: {}", e);
            }
            if let Err(e) = store.save_token(&token_id, self.token_registry.get(&token_id).unwrap())
            {
                tracing::warn!("Failed to persist token record after burn: {}", e);
            }
        }

        Ok(())
    }

    /// Apply a token burn from a peer block.
    /// Warns on insufficient balance (best-effort, like peer transfers).
    pub fn apply_peer_token_burn(
        &mut self,
        token_id: TokenId,
        burner: Address,
        burner_pubkey: PublicKey,
        amount: Amount,
    ) -> Result<(), NornError> {
        self.auto_register_with_pubkey(burner, burner_pubkey);

        if let Some(state) = self.thread_states.get(&burner) {
            if state.has_balance(&token_id, amount) {
                let state = self.thread_states.get_mut(&burner).unwrap();
                state.debit(&token_id, amount);

                if let Some(meta) = self.thread_meta.get_mut(&burner) {
                    meta.state_hash = norn_thread::state::compute_state_hash(
                        self.thread_states.get(&burner).unwrap(),
                    );
                }

                // Update SMT.
                self.update_smt(&burner, &token_id);
            } else {
                tracing::warn!(
                    "peer token burn: burner {} has insufficient balance for {}",
                    hex::encode(burner),
                    amount,
                );
            }
        }

        // Update supply.
        if let Some(record) = self.token_registry.get_mut(&token_id) {
            record.current_supply = record.current_supply.saturating_sub(amount);
        }

        Ok(())
    }

    /// Get a token record by ID.
    pub fn get_token(&self, token_id: &TokenId) -> Option<&TokenRecord> {
        self.token_registry.get(token_id)
    }

    /// Get a token ID by symbol.
    pub fn get_token_by_symbol(&self, symbol: &str) -> Option<(&TokenId, &TokenRecord)> {
        self.symbol_index
            .get(symbol)
            .and_then(|id| self.token_registry.get(id).map(|r| (id, r)))
    }

    /// List all tokens (for RPC).
    pub fn list_tokens(&self) -> Vec<(&TokenId, &TokenRecord)> {
        self.token_registry.iter().collect()
    }

    /// Iterate over registered tokens for WeaveEngine seeding.
    pub fn registered_tokens(&self) -> impl Iterator<Item = (&TokenId, &TokenRecord)> {
        self.token_registry.iter()
    }

    /// Seed a token into the registry (used during state rebuild).
    pub fn seed_token(&mut self, token_id: TokenId, record: TokenRecord) {
        self.symbol_index.insert(record.symbol.clone(), token_id);
        self.token_registry.insert(token_id, record);
    }

    // ── Loom Operations ──────────────────────────────────────────────────

    /// Deploy a loom (solo path — deducts fee).
    pub fn deploy_loom(
        &mut self,
        loom_id: LoomId,
        name: &str,
        operator: PublicKey,
        operator_address: Address,
        timestamp: u64,
    ) -> Result<(), NornError> {
        // Deduct deploy fee from operator (warn but don't fail if insufficient).
        self.debit_fee(operator_address, LOOM_DEPLOY_FEE);

        let record = LoomRecord {
            name: name.to_string(),
            operator,
            max_participants: 1000,
            min_participants: 1,
            active: true,
            deployed_at: timestamp,
        };

        self.loom_registry.insert(loom_id, record.clone());

        // Persist.
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_loom(&loom_id, &record) {
                tracing::warn!("failed to persist loom: {}", e);
            }
        }

        Ok(())
    }

    /// Apply a peer loom deploy (skips fee deduction).
    pub fn apply_peer_loom_deploy(
        &mut self,
        loom_id: LoomId,
        name: &str,
        operator: PublicKey,
        timestamp: u64,
    ) {
        if self.loom_registry.contains_key(&loom_id) {
            tracing::debug!(
                loom_id = %hex::encode(loom_id),
                "skipping duplicate peer loom deploy"
            );
            return;
        }

        let record = LoomRecord {
            name: name.to_string(),
            operator,
            max_participants: 1000,
            min_participants: 1,
            active: true,
            deployed_at: timestamp,
        };

        self.loom_registry.insert(loom_id, record.clone());

        // Persist.
        if let Some(ref store) = self.state_store {
            if let Err(e) = store.save_loom(&loom_id, &record) {
                tracing::warn!("failed to persist peer loom: {}", e);
            }
        }
    }

    /// Get a loom record by ID.
    pub fn get_loom(&self, loom_id: &LoomId) -> Option<&LoomRecord> {
        self.loom_registry.get(loom_id)
    }

    /// List all looms (for RPC).
    pub fn list_looms(&self) -> Vec<(&LoomId, &LoomRecord)> {
        self.loom_registry.iter().collect()
    }

    /// Iterate over registered looms for WeaveEngine seeding.
    pub fn registered_looms(&self) -> impl Iterator<Item = &LoomId> {
        self.loom_registry.keys()
    }

    /// Seed a loom into the registry (used during state rebuild).
    pub fn seed_loom(&mut self, loom_id: LoomId, record: LoomRecord) {
        self.loom_registry.insert(loom_id, record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::constants::ONE_NORN;

    fn test_address(byte: u8) -> Address {
        [byte; 20]
    }

    fn test_pubkey(byte: u8) -> PublicKey {
        [byte; 32]
    }

    #[test]
    fn test_register_and_check() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        assert!(!sm.is_registered(&addr));
        sm.register_thread(addr, test_pubkey(1));
        assert!(sm.is_registered(&addr));
    }

    #[test]
    fn test_double_register_is_noop() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 100).unwrap();
        sm.register_thread(addr, test_pubkey(2)); // should not reset
        assert_eq!(sm.get_balance(&addr, &NATIVE_TOKEN_ID), 100);
    }

    #[test]
    fn test_credit_and_balance() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 1000).unwrap();
        assert_eq!(sm.get_balance(&addr, &NATIVE_TOKEN_ID), 1000);
    }

    #[test]
    fn test_credit_unregistered_fails() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        assert!(sm.credit(addr, NATIVE_TOKEN_ID, 1000).is_err());
    }

    #[test]
    fn test_transfer() {
        let mut sm = StateManager::new();
        let alice = test_address(1);
        let bob = test_address(2);
        sm.register_thread(alice, test_pubkey(1));
        sm.register_thread(bob, test_pubkey(2));
        sm.credit(alice, NATIVE_TOKEN_ID, 1000).unwrap();

        sm.apply_transfer(alice, bob, NATIVE_TOKEN_ID, 400, [0u8; 32], None, 1000)
            .unwrap();

        assert_eq!(sm.get_balance(&alice, &NATIVE_TOKEN_ID), 600);
        assert_eq!(sm.get_balance(&bob, &NATIVE_TOKEN_ID), 400);
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let mut sm = StateManager::new();
        let alice = test_address(1);
        let bob = test_address(2);
        sm.register_thread(alice, test_pubkey(1));
        sm.register_thread(bob, test_pubkey(2));
        sm.credit(alice, NATIVE_TOKEN_ID, 100).unwrap();

        let result = sm.apply_transfer(alice, bob, NATIVE_TOKEN_ID, 200, [0u8; 32], None, 1000);
        assert!(result.is_err());
        // Balances unchanged
        assert_eq!(sm.get_balance(&alice, &NATIVE_TOKEN_ID), 100);
        assert_eq!(sm.get_balance(&bob, &NATIVE_TOKEN_ID), 0);
    }

    #[test]
    fn test_transfer_zero_amount() {
        let mut sm = StateManager::new();
        let alice = test_address(1);
        let bob = test_address(2);
        sm.register_thread(alice, test_pubkey(1));
        sm.register_thread(bob, test_pubkey(2));

        let result = sm.apply_transfer(alice, bob, NATIVE_TOKEN_ID, 0, [0u8; 32], None, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_history() {
        let mut sm = StateManager::new();
        let alice = test_address(1);
        let bob = test_address(2);
        sm.register_thread(alice, test_pubkey(1));
        sm.register_thread(bob, test_pubkey(2));
        sm.credit(alice, NATIVE_TOKEN_ID, 1000).unwrap();

        sm.apply_transfer(alice, bob, NATIVE_TOKEN_ID, 100, [1u8; 32], None, 1000)
            .unwrap();
        sm.apply_transfer(alice, bob, NATIVE_TOKEN_ID, 200, [2u8; 32], None, 2000)
            .unwrap();

        let history = sm.get_history(&alice, 10, 0);
        assert_eq!(history.len(), 2);
        // Most recent first
        assert_eq!(history[0].amount, 200);
        assert_eq!(history[1].amount, 100);

        let bob_history = sm.get_history(&bob, 10, 0);
        assert_eq!(bob_history.len(), 2);
    }

    #[test]
    fn test_auto_register() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        assert!(!sm.is_registered(&addr));
        sm.auto_register_if_needed(addr);
        assert!(sm.is_registered(&addr));
    }

    #[test]
    fn test_archive_and_get_block() {
        let mut sm = StateManager::new();
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
            stake_operations: vec![],
            stake_operations_root: [0u8; 32],
            state_root: [0u8; 32],
            timestamp: 1000,
            proposer: [0u8; 32],
            validator_signatures: vec![],
        };
        sm.archive_block(block);
        assert!(sm.get_block(1).is_some());
        assert!(sm.get_block(2).is_none());
        assert_eq!(sm.latest_block_height(), 1);
    }

    #[test]
    fn test_faucet_log() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 100).unwrap();
        sm.log_faucet_credit(addr, 100, 1000);

        let history = sm.get_history(&addr, 10, 0);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].from, [0u8; 20]);
        assert_eq!(history[0].memo, Some(b"faucet".to_vec()));
    }

    // ─── Name Registry Tests ────────────────────────────────────────────────

    #[test]
    fn test_register_name() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 2 * ONE_NORN).unwrap();

        sm.register_name("alice", addr, 1000).unwrap();
        let record = sm.resolve_name("alice").unwrap();
        assert_eq!(record.owner, addr);
        assert_eq!(record.registered_at, 1000);
        assert_eq!(record.fee_paid, ONE_NORN);
    }

    #[test]
    fn test_register_name_duplicate_rejected() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 5 * ONE_NORN).unwrap();

        sm.register_name("alice", addr, 1000).unwrap();
        let result = sm.register_name("alice", addr, 2000);
        assert!(matches!(result, Err(NornError::NameAlreadyRegistered(_))));
    }

    #[test]
    fn test_name_validation_rules() {
        assert!(validate_name("ab").is_err()); // too short
        assert!(validate_name("a".repeat(33).as_str()).is_err()); // too long
        assert!(validate_name("-alice").is_err()); // leading hyphen
        assert!(validate_name("alice-").is_err()); // trailing hyphen
        assert!(validate_name("Alice").is_err()); // uppercase
        assert!(validate_name("al ice").is_err()); // space
        assert!(validate_name("al_ice").is_err()); // underscore

        assert!(validate_name("abc").is_ok());
        assert!(validate_name("alice").is_ok());
        assert!(validate_name("my-name").is_ok());
        assert!(validate_name("user123").is_ok());
        assert!(validate_name("a".repeat(32).as_str()).is_ok());
    }

    #[test]
    fn test_register_name_fee_deduction() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 5 * ONE_NORN).unwrap();

        sm.register_name("alice", addr, 1000).unwrap();
        assert_eq!(sm.get_balance(&addr, &NATIVE_TOKEN_ID), 4 * ONE_NORN);
    }

    #[test]
    fn test_register_name_insufficient_balance() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, ONE_NORN / 2).unwrap(); // less than 1 NORN

        let result = sm.register_name("alice", addr, 1000);
        assert!(matches!(result, Err(NornError::InsufficientBalance { .. })));
    }

    #[test]
    fn test_name_reverse_lookup() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 5 * ONE_NORN).unwrap();

        sm.register_name("alice", addr, 1000).unwrap();
        sm.register_name("bob", addr, 2000).unwrap();

        let names = sm.names_for_address(&addr);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"alice"));
        assert!(names.contains(&"bob"));

        // Unknown address returns empty
        let unknown = test_address(99);
        assert!(sm.names_for_address(&unknown).is_empty());
    }

    #[test]
    fn test_resolve_nonexistent_name() {
        let sm = StateManager::new();
        assert!(sm.resolve_name("nonexistent").is_none());
    }

    // ─── Supply Cap Tests ──────────────────────────────────────────────────

    #[test]
    fn test_total_supply_tracking() {
        let mut sm = StateManager::new();
        assert_eq!(sm.total_supply(), 0);

        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));
        sm.credit(addr, NATIVE_TOKEN_ID, 1000).unwrap();
        assert_eq!(sm.total_supply(), 1000);

        let addr2 = test_address(2);
        sm.register_thread(addr2, test_pubkey(2));
        sm.credit(addr2, NATIVE_TOKEN_ID, 500).unwrap();
        assert_eq!(sm.total_supply(), 1500);
    }

    #[test]
    fn test_supply_cap_enforcement() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));

        // Credit near the cap should succeed.
        sm.credit(addr, NATIVE_TOKEN_ID, MAX_SUPPLY - 100).unwrap();
        assert_eq!(sm.total_supply(), MAX_SUPPLY - 100);

        // Credit that would exceed should fail.
        let result = sm.credit(addr, NATIVE_TOKEN_ID, 200);
        assert!(matches!(result, Err(NornError::SupplyCapExceeded { .. })));

        // Total supply unchanged after failure.
        assert_eq!(sm.total_supply(), MAX_SUPPLY - 100);

        // Credit up to exactly MAX_SUPPLY should succeed.
        sm.credit(addr, NATIVE_TOKEN_ID, 100).unwrap();
        assert_eq!(sm.total_supply(), MAX_SUPPLY);
    }

    #[test]
    fn test_non_native_token_no_supply_cap() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));

        // Non-native tokens are not capped.
        let custom_token = [42u8; 32];
        sm.credit(addr, custom_token, u128::MAX / 2).unwrap();
        assert_eq!(sm.get_balance(&addr, &custom_token), u128::MAX / 2);
    }

    #[test]
    fn test_from_parts_computes_total_supply() {
        let mut states = HashMap::new();
        let addr1 = test_address(1);
        let addr2 = test_address(2);

        let mut s1 = ThreadState::new();
        s1.credit(NATIVE_TOKEN_ID, 500).unwrap();
        states.insert(addr1, s1);

        let mut s2 = ThreadState::new();
        s2.credit(NATIVE_TOKEN_ID, 300).unwrap();
        states.insert(addr2, s2);

        let sm = StateManager::from_parts(
            states,
            HashMap::new(),
            Vec::new(),
            Vec::new(),
            HashMap::new(),
            HashMap::new(),
        );
        assert_eq!(sm.total_supply(), 800);
    }

    // ── Token tests ───────────────────────────────────

    #[test]
    fn test_create_token() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        let creator_pk = test_pubkey(1);
        sm.register_thread(creator, creator_pk);
        sm.credit(creator, NATIVE_TOKEN_ID, 100 * ONE_NORN).unwrap();

        let token_id = sm
            .create_token("Test", "TST", 8, 1_000_000, 100, creator, 12345)
            .unwrap();

        assert_ne!(token_id, [0u8; 32]);
        let record = sm.get_token(&token_id).expect("token should exist");
        assert_eq!(record.name, "Test");
        assert_eq!(record.symbol, "TST");
        assert_eq!(record.decimals, 8);
        assert_eq!(record.max_supply, 1_000_000);
        assert_eq!(record.current_supply, 100);
        assert_eq!(record.creator, creator);

        // Creator should have initial supply credited.
        assert_eq!(sm.get_balance(&creator, &token_id), 100);

        // Fee should be deducted.
        let expected_balance = 100 * ONE_NORN - norn_types::token::TOKEN_CREATION_FEE;
        assert_eq!(sm.get_balance(&creator, &NATIVE_TOKEN_ID), expected_balance);
    }

    #[test]
    fn test_create_token_duplicate_symbol() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        sm.register_thread(creator, test_pubkey(1));
        sm.credit(creator, NATIVE_TOKEN_ID, 200 * ONE_NORN).unwrap();

        sm.create_token("Test1", "TST", 8, 0, 0, creator, 100)
            .unwrap();
        let result = sm.create_token("Test2", "TST", 8, 0, 0, creator, 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_token_insufficient_balance() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        sm.register_thread(creator, test_pubkey(1));
        sm.credit(creator, NATIVE_TOKEN_ID, ONE_NORN).unwrap(); // Only 1 NORN, need 10

        let result = sm.create_token("Test", "TST", 8, 0, 0, creator, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_mint_token() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        let recipient = test_address(2);
        sm.register_thread(creator, test_pubkey(1));
        sm.register_thread(recipient, test_pubkey(2));
        sm.credit(creator, NATIVE_TOKEN_ID, 100 * ONE_NORN).unwrap();

        let token_id = sm
            .create_token("Test", "TST", 8, 10_000, 0, creator, 100)
            .unwrap();

        sm.mint_token(token_id, recipient, 500).unwrap();
        assert_eq!(sm.get_balance(&recipient, &token_id), 500);

        let record = sm.get_token(&token_id).unwrap();
        assert_eq!(record.current_supply, 500);
    }

    #[test]
    fn test_mint_token_supply_cap() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        sm.register_thread(creator, test_pubkey(1));
        sm.credit(creator, NATIVE_TOKEN_ID, 100 * ONE_NORN).unwrap();

        let token_id = sm
            .create_token("Test", "TST", 8, 100, 50, creator, 100)
            .unwrap();

        // Try to mint more than remaining capacity (50 remaining).
        let result = sm.mint_token(token_id, creator, 51);
        assert!(result.is_err());

        // Mint exactly remaining capacity.
        sm.mint_token(token_id, creator, 50).unwrap();
        assert_eq!(sm.get_balance(&creator, &token_id), 100);
    }

    #[test]
    fn test_mint_token_nonexistent() {
        let mut sm = StateManager::new();
        let addr = test_address(1);
        sm.register_thread(addr, test_pubkey(1));

        let fake_token = [99u8; 32];
        let result = sm.mint_token(fake_token, addr, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_burn_token() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        sm.register_thread(creator, test_pubkey(1));
        sm.credit(creator, NATIVE_TOKEN_ID, 100 * ONE_NORN).unwrap();

        let token_id = sm
            .create_token("Test", "TST", 8, 0, 1000, creator, 100)
            .unwrap();

        sm.burn_token(token_id, creator, 300).unwrap();
        assert_eq!(sm.get_balance(&creator, &token_id), 700);

        let record = sm.get_token(&token_id).unwrap();
        assert_eq!(record.current_supply, 700);
    }

    #[test]
    fn test_get_token_by_symbol() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        sm.register_thread(creator, test_pubkey(1));
        sm.credit(creator, NATIVE_TOKEN_ID, 100 * ONE_NORN).unwrap();

        let token_id = sm
            .create_token("Test", "TST", 8, 0, 0, creator, 100)
            .unwrap();

        let (found_id, found_record) = sm.get_token_by_symbol("TST").unwrap();
        assert_eq!(*found_id, token_id);
        assert_eq!(found_record.symbol, "TST");

        assert!(sm.get_token_by_symbol("NONEXISTENT").is_none());
    }

    #[test]
    fn test_list_tokens() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        sm.register_thread(creator, test_pubkey(1));
        sm.credit(creator, NATIVE_TOKEN_ID, 200 * ONE_NORN).unwrap();

        sm.create_token("Alpha", "ALPHA", 18, 0, 0, creator, 100)
            .unwrap();
        sm.create_token("Beta", "BETA", 8, 0, 0, creator, 200)
            .unwrap();

        let tokens = sm.list_tokens();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_apply_peer_token_creation() {
        let mut sm = StateManager::new();
        let creator = test_address(1);
        sm.register_thread(creator, test_pubkey(1));
        sm.credit(creator, NATIVE_TOKEN_ID, 100 * ONE_NORN).unwrap();
        let original_balance = sm.get_balance(&creator, &NATIVE_TOKEN_ID);

        let token_id = sm
            .apply_peer_token_creation(
                "Peer Token",
                "PTK",
                18,
                0,
                500,
                creator,
                test_pubkey(1),
                100,
            )
            .unwrap();

        // Peer path should NOT deduct fee.
        assert_eq!(sm.get_balance(&creator, &NATIVE_TOKEN_ID), original_balance);

        // But should credit initial supply.
        assert_eq!(sm.get_balance(&creator, &token_id), 500);

        let record = sm.get_token(&token_id).unwrap();
        assert_eq!(record.symbol, "PTK");
        assert_eq!(record.current_supply, 500);
    }
}
