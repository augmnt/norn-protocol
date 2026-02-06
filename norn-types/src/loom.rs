use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

/// Configuration for a loom.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomConfig {
    /// Unique identifier for this loom.
    pub loom_id: LoomId,
    /// Human-readable name.
    pub name: String,
    /// Maximum number of participants.
    pub max_participants: usize,
    /// Minimum number of participants for the loom to be active.
    pub min_participants: usize,
    /// Tokens accepted by this loom.
    pub accepted_tokens: Vec<TokenId>,
    /// Opaque loom-specific configuration data.
    pub config_data: Vec<u8>,
}

/// A participant in a loom.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Participant {
    /// Participant's public key.
    pub pubkey: PublicKey,
    /// Participant's address (thread ID).
    pub address: Address,
    /// Timestamp when the participant joined.
    pub joined_at: Timestamp,
    /// Whether the participant is currently active.
    pub active: bool,
}

/// A loom registration request.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomRegistration {
    /// The loom configuration.
    pub config: LoomConfig,
    /// The loom operator's public key.
    pub operator: PublicKey,
    /// Timestamp of registration.
    pub timestamp: Timestamp,
    /// Signature by the operator.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A loom instance with its current state.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Loom {
    /// Loom configuration.
    pub config: LoomConfig,
    /// Loom operator's public key.
    pub operator: PublicKey,
    /// Current participants.
    pub participants: Vec<Participant>,
    /// Hash of the current loom state.
    pub state_hash: Hash,
    /// Current loom state version.
    pub version: Version,
    /// Whether the loom is currently active.
    pub active: bool,
    /// Timestamp of last state update.
    pub last_updated: Timestamp,
}

/// Deployed loom bytecode (Wasm module).
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomBytecode {
    /// The loom this bytecode belongs to.
    pub loom_id: LoomId,
    /// Hash of the Wasm bytecode.
    pub wasm_hash: Hash,
    /// The Wasm bytecode itself.
    pub bytecode: Vec<u8>,
}

/// A loom state transition record.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomStateTransition {
    /// The loom ID.
    pub loom_id: LoomId,
    /// Hash of the state before the transition.
    pub prev_state_hash: Hash,
    /// Hash of the state after the transition.
    pub new_state_hash: Hash,
    /// Inputs to the transition.
    pub inputs: Vec<u8>,
    /// Outputs of the transition.
    pub outputs: Vec<u8>,
}

/// A challenge to a loom state transition.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomChallenge {
    /// The loom ID.
    pub loom_id: LoomId,
    /// The disputed state transition.
    pub transition: LoomStateTransition,
    /// The challenger's public key.
    pub challenger: PublicKey,
    /// Timestamp of the challenge.
    pub timestamp: Timestamp,
    /// Signature by the challenger.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}
