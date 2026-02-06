use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

/// The fixed-size header committed to the weave for each thread.
/// Contains the essential state needed to verify knot chains.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ThreadHeader {
    /// The thread's unique ID (same as the creator's address).
    pub thread_id: ThreadId,
    /// The creator's public key.
    pub owner: PublicKey,
    /// Current version counter (incremented per knot involving this thread).
    pub version: Version,
    /// Hash of the current thread state (balances, assets, looms).
    pub state_hash: Hash,
    /// Hash of the last knot applied to this thread (zeros if none).
    pub last_knot_hash: Hash,
    /// Hash of the previous committed header (zeros for genesis).
    pub prev_header_hash: Hash,
    /// Timestamp of this commitment.
    pub timestamp: Timestamp,
    /// Signature by the thread owner over this header.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// The full mutable state of a thread (not committed directly — only its hash).
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ThreadState {
    /// Balances per token. Key: TokenId, Value: Amount.
    pub balances: BTreeMap<TokenId, Amount>,
    /// Assets held (e.g., NFTs or other non-fungible items).
    pub assets: BTreeMap<TokenId, Vec<u8>>,
    /// Loom memberships. Key: LoomId, Value: loom-specific data.
    pub looms: BTreeMap<LoomId, Vec<u8>>,
    /// Replay-protection nonce, incremented with each knot.
    pub nonce: u64,
}

impl ThreadState {
    /// Create a new empty thread state.
    pub fn new() -> Self {
        Self {
            balances: BTreeMap::new(),
            assets: BTreeMap::new(),
            looms: BTreeMap::new(),
            nonce: 0,
        }
    }

    /// Get the balance for a specific token.
    pub fn balance(&self, token_id: &TokenId) -> Amount {
        self.balances.get(token_id).copied().unwrap_or(0)
    }

    /// Check if the thread has at least the specified balance.
    pub fn has_balance(&self, token_id: &TokenId, amount: Amount) -> bool {
        self.balance(token_id) >= amount
    }

    /// Credit tokens to this thread. Returns error on overflow.
    pub fn credit(
        &mut self,
        token_id: TokenId,
        amount: Amount,
    ) -> Result<(), crate::error::NornError> {
        let entry = self.balances.entry(token_id).or_insert(0);
        *entry = entry
            .checked_add(amount)
            .ok_or(crate::error::NornError::BalanceOverflow)?;
        Ok(())
    }

    /// Debit tokens from this thread. Returns false if insufficient balance.
    pub fn debit(&mut self, token_id: &TokenId, amount: Amount) -> bool {
        if let Some(balance) = self.balances.get_mut(token_id) {
            if *balance >= amount {
                *balance -= amount;
                if *balance == 0 {
                    self.balances.remove(token_id);
                }
                return true;
            }
        }
        false
    }
}

impl Default for ThreadState {
    fn default() -> Self {
        Self::new()
    }
}
