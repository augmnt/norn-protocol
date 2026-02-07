use std::collections::HashMap;

use norn_types::constants::ONE_NORN;
use norn_types::error::NornError;
use norn_types::primitives::{Address, Amount, Hash, PublicKey, TokenId, NATIVE_TOKEN_ID};
use norn_types::thread::ThreadState;
use norn_types::weave::WeaveBlock;

/// Fee for registering a name (1 NORN, burned).
pub const NAME_REGISTRATION_FEE: Amount = ONE_NORN;

/// A record of a registered name.
#[derive(Debug, Clone)]
pub struct NameRecord {
    pub owner: Address,
    pub registered_at: u64,
    pub fee_paid: Amount,
}

/// Validate a name: lowercase alphanumeric + hyphens, 3-32 chars, no leading/trailing hyphens.
pub fn validate_name(name: &str) -> Result<(), NornError> {
    if name.len() < 3 || name.len() > 32 {
        return Err(NornError::InvalidName(format!(
            "name must be 3-32 characters, got {}",
            name.len()
        )));
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err(NornError::InvalidName(
            "name must not start or end with a hyphen".to_string(),
        ));
    }
    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            return Err(NornError::InvalidName(format!(
                "name must be lowercase alphanumeric or hyphens, found '{}'",
                c
            )));
        }
    }
    Ok(())
}

/// Metadata tracked per thread beyond its ThreadState.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ThreadMeta {
    pub owner: PublicKey,
    pub version: u64,
    pub nonce: u64,
    pub state_hash: Hash,
    pub last_commit_hash: Hash,
}

/// A record of a token transfer (for history queries).
#[derive(Debug, Clone)]
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

/// Node-side state manager that tracks balances, history, and blocks
/// alongside the WeaveEngine's consensus-level tracking.
pub struct StateManager {
    thread_states: HashMap<Address, ThreadState>,
    thread_meta: HashMap<Address, ThreadMeta>,
    transfer_log: Vec<TransferRecord>,
    block_archive: Vec<WeaveBlock>,
    name_registry: HashMap<String, NameRecord>,
    address_names: HashMap<Address, Vec<String>>,
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
        }
    }

    /// Register a thread with its owner public key.
    pub fn register_thread(&mut self, address: Address, pubkey: PublicKey) {
        if self.thread_states.contains_key(&address) {
            return;
        }
        let state_hash = norn_thread::state::compute_state_hash(&ThreadState::new());
        self.thread_states.insert(address, ThreadState::new());
        self.thread_meta.insert(
            address,
            ThreadMeta {
                owner: pubkey,
                version: 0,
                nonce: 0,
                state_hash,
                last_commit_hash: [0u8; 32],
            },
        );
    }

    /// Check if an address is registered.
    pub fn is_registered(&self, address: &Address) -> bool {
        self.thread_states.contains_key(address)
    }

    /// Auto-register a thread if not already present (for transfer recipients).
    /// Uses a zero pubkey since we don't know the recipient's key.
    pub fn auto_register_if_needed(&mut self, address: Address) {
        if !self.is_registered(&address) {
            self.register_thread(address, [0u8; 32]);
        }
    }

    /// Credit tokens to an address (e.g., faucet).
    pub fn credit(
        &mut self,
        address: Address,
        token_id: TokenId,
        amount: Amount,
    ) -> Result<(), NornError> {
        let state = self
            .thread_states
            .get_mut(&address)
            .ok_or(NornError::ThreadNotFound(address))?;
        state.credit(token_id, amount)?;

        // Update state hash in meta
        if let Some(meta) = self.thread_meta.get_mut(&address) {
            meta.state_hash = norn_thread::state::compute_state_hash(state);
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

        // Log the transfer
        self.transfer_log.push(TransferRecord {
            knot_id,
            from,
            to,
            token_id,
            amount,
            memo,
            timestamp,
            block_height: None,
        });

        Ok(())
    }

    /// Get balance for an address and token.
    pub fn get_balance(&self, address: &Address, token_id: &TokenId) -> Amount {
        self.thread_states
            .get(address)
            .map(|s| s.balance(token_id))
            .unwrap_or(0)
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

    /// Archive a produced block.
    pub fn archive_block(&mut self, block: WeaveBlock) {
        self.block_archive.push(block);
    }

    /// Get a block by height.
    pub fn get_block(&self, height: u64) -> Option<&WeaveBlock> {
        self.block_archive.iter().find(|b| b.height == height)
    }

    /// Get the latest block height.
    #[allow(dead_code)]
    pub fn latest_block_height(&self) -> u64 {
        self.block_archive.last().map(|b| b.height).unwrap_or(0)
    }

    /// Log a faucet credit as a transfer record (from zero-address).
    pub fn log_faucet_credit(&mut self, address: Address, amount: Amount, timestamp: u64) {
        self.transfer_log.push(TransferRecord {
            knot_id: [0u8; 32],
            from: [0u8; 20], // zero address = faucet
            to: address,
            token_id: NATIVE_TOKEN_ID,
            amount,
            memo: Some(b"faucet".to_vec()),
            timestamp,
            block_height: None,
        });
    }

    /// Register a name for an address. Validates the name, checks uniqueness,
    /// deducts the registration fee (burned), and records the name.
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

        // Update state hash.
        if let Some(meta) = self.thread_meta.get_mut(&owner) {
            meta.state_hash =
                norn_thread::state::compute_state_hash(self.thread_states.get(&owner).unwrap());
        }

        // Log the fee burn as a transfer to the zero address.
        self.transfer_log.push(TransferRecord {
            knot_id: [0u8; 32],
            from: owner,
            to: [0u8; 20], // burn address
            token_id: NATIVE_TOKEN_ID,
            amount: NAME_REGISTRATION_FEE,
            memo: Some(format!("name registration: {}", name).into_bytes()),
            timestamp,
            block_height: None,
        });

        // Record the name.
        self.name_registry.insert(
            name.to_string(),
            NameRecord {
                owner,
                registered_at: timestamp,
                fee_paid: NAME_REGISTRATION_FEE,
            },
        );
        self.address_names
            .entry(owner)
            .or_default()
            .push(name.to_string());

        Ok(())
    }

    /// Resolve a name to its record.
    pub fn resolve_name(&self, name: &str) -> Option<&NameRecord> {
        self.name_registry.get(name)
    }

    /// Get all names owned by an address.
    pub fn names_for_address(&self, address: &Address) -> Vec<&str> {
        self.address_names
            .get(address)
            .map(|names| names.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            fraud_proofs: vec![],
            fraud_proofs_root: [0u8; 32],
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
}
