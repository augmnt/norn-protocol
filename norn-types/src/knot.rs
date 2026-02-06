use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

/// The type of operation a knot performs.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum KnotType {
    /// Simple two-party token transfer.
    Transfer,
    /// Multi-party transfer (up to MAX_MULTI_TRANSFERS).
    MultiTransfer,
    /// Interaction with a loom (deposit, withdraw, state update).
    LoomInteraction,
}

/// Snapshot of a participant's thread state before or after a knot.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ParticipantState {
    /// The participant's thread ID.
    pub thread_id: ThreadId,
    /// The participant's public key.
    pub pubkey: PublicKey,
    /// Version number of the thread at this point.
    pub version: Version,
    /// Hash of the thread state at this point.
    pub state_hash: Hash,
}

/// A single transfer between two parties.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct TransferPayload {
    /// Token being transferred.
    pub token_id: TokenId,
    /// Amount being transferred.
    pub amount: Amount,
    /// Sender's address (thread ID).
    pub from: Address,
    /// Recipient's address (thread ID).
    pub to: Address,
    /// Optional memo (max MAX_MEMO_SIZE bytes).
    pub memo: Option<Vec<u8>>,
}

/// Multiple transfers in a single knot.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MultiTransferPayload {
    /// List of individual transfers.
    pub transfers: Vec<TransferPayload>,
}

/// Loom interaction types.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum LoomInteractionType {
    /// Deposit tokens into a loom.
    Deposit,
    /// Withdraw tokens from a loom.
    Withdraw,
    /// Update loom state.
    StateUpdate,
}

/// Payload for a loom interaction knot.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomInteractionPayload {
    /// The loom being interacted with.
    pub loom_id: LoomId,
    /// Type of interaction.
    pub interaction_type: LoomInteractionType,
    /// Token involved (for deposits/withdrawals).
    pub token_id: Option<TokenId>,
    /// Amount involved (for deposits/withdrawals).
    pub amount: Option<Amount>,
    /// Opaque loom-specific data.
    pub data: Vec<u8>,
}

/// The payload of a knot — varies by knot type.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum KnotPayload {
    Transfer(TransferPayload),
    MultiTransfer(MultiTransferPayload),
    LoomInteraction(LoomInteractionPayload),
}

/// A knot is the fundamental unit of state transition in Norn.
/// It records a bilateral or multilateral agreement between thread participants.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Knot {
    /// Unique identifier: BLAKE3(all fields except signatures).
    pub id: KnotId,
    /// The type of this knot.
    pub knot_type: KnotType,
    /// Timestamp when the knot was created.
    pub timestamp: Timestamp,
    /// Optional expiry timestamp.
    pub expiry: Option<Timestamp>,
    /// Each participant's state BEFORE the knot.
    pub before_states: Vec<ParticipantState>,
    /// Each participant's state AFTER the knot.
    pub after_states: Vec<ParticipantState>,
    /// The operation payload.
    pub payload: KnotPayload,
    /// Signatures from all participants (one per participant, in order).
    #[serde(with = "crate::primitives::serde_sig_vec")]
    pub signatures: Vec<Signature>,
}
