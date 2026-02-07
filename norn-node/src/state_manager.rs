use std::collections::HashMap;

use norn_types::error::NornError;
use norn_types::primitives::{Address, Amount, Hash, PublicKey, TokenId, NATIVE_TOKEN_ID};
use norn_types::thread::ThreadState;
use norn_types::weave::WeaveBlock;

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
}
