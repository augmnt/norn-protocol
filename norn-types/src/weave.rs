use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::fraud::FraudProofSubmission;
use crate::primitives::*;

/// A commitment update submitted by a thread to the weave.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct CommitmentUpdate {
    /// The thread submitting the commitment.
    pub thread_id: ThreadId,
    /// The thread owner's public key.
    pub owner: PublicKey,
    /// New version number after this commitment.
    pub version: Version,
    /// Hash of the new thread state.
    pub state_hash: Hash,
    /// Hash of the previous commitment (zeros for genesis).
    pub prev_commitment_hash: Hash,
    /// Number of knots since the last commitment.
    pub knot_count: u64,
    /// Timestamp of this commitment.
    pub timestamp: Timestamp,
    /// Signature by the thread owner.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A thread registration on the weave.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Registration {
    /// The thread being registered.
    pub thread_id: ThreadId,
    /// The thread owner's public key.
    pub owner: PublicKey,
    /// Initial state hash.
    pub initial_state_hash: Hash,
    /// Timestamp of registration.
    pub timestamp: Timestamp,
    /// Signature by the thread owner.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A loom anchor posted to the weave for cross-thread coordination.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomAnchor {
    /// The loom being anchored.
    pub loom_id: LoomId,
    /// Hash of the loom's current state.
    pub state_hash: Hash,
    /// Block height at which this anchor was created.
    pub block_height: u64,
    /// Timestamp of this anchor.
    pub timestamp: Timestamp,
    /// Signature by the loom operator.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A name registration on the weave.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct NameRegistration {
    /// The name being registered.
    pub name: String,
    /// The owner's address.
    pub owner: Address,
    /// The owner's public key (needed for signature verification).
    pub owner_pubkey: PublicKey,
    /// Timestamp of registration.
    pub timestamp: Timestamp,
    /// Fee paid for registration.
    pub fee_paid: Amount,
    /// Signature by the owner.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A transfer record included in a weave block for cross-node balance sync.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct BlockTransfer {
    pub from: Address,
    pub to: Address,
    pub token_id: TokenId,
    pub amount: Amount,
    pub memo: Option<Vec<u8>>,
    pub knot_id: Hash,
    pub timestamp: u64,
}

/// A validator's signature on a weave block.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ValidatorSignature {
    /// The validator's public key.
    pub validator: PublicKey,
    /// Signature over the block hash.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// Validator information.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Validator {
    /// Validator's public key.
    pub pubkey: PublicKey,
    /// Validator's address.
    pub address: Address,
    /// Stake amount.
    pub stake: Amount,
    /// Whether the validator is currently active.
    pub active: bool,
}

/// A block in the weave — the global ordering layer.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct WeaveBlock {
    /// Block height.
    pub height: u64,
    /// Hash of this block.
    pub hash: Hash,
    /// Hash of the previous block.
    pub prev_hash: Hash,
    /// Merkle root of all commitment updates in this block.
    pub commitments_root: Hash,
    /// Merkle root of all registrations in this block.
    pub registrations_root: Hash,
    /// Merkle root of all loom anchors in this block.
    pub anchors_root: Hash,
    /// Commitment updates included in this block.
    pub commitments: Vec<CommitmentUpdate>,
    /// Thread registrations included in this block.
    pub registrations: Vec<Registration>,
    /// Loom anchors included in this block.
    pub anchors: Vec<LoomAnchor>,
    /// Name registrations included in this block.
    pub name_registrations: Vec<NameRegistration>,
    /// Merkle root of all name registrations in this block.
    pub name_registrations_root: Hash,
    /// Fraud proof submissions included in this block.
    pub fraud_proofs: Vec<FraudProofSubmission>,
    /// Merkle root of all fraud proofs in this block.
    pub fraud_proofs_root: Hash,
    /// Transfers included in this block (for cross-node balance sync).
    pub transfers: Vec<BlockTransfer>,
    /// Merkle root of all transfers in this block.
    pub transfers_root: Hash,
    /// Block timestamp.
    pub timestamp: Timestamp,
    /// Block proposer's public key.
    pub proposer: PublicKey,
    /// Validator signatures.
    pub validator_signatures: Vec<ValidatorSignature>,
}

/// Global weave state tracking.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct WeaveState {
    /// Current block height.
    pub height: u64,
    /// Hash of the latest block.
    pub latest_hash: Hash,
    /// Merkle root of all registered threads.
    pub threads_root: Hash,
    /// Total number of registered threads.
    pub thread_count: u64,
    /// Current fee state.
    pub fee_state: FeeState,
}

/// Fee parameters for the weave.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct FeeState {
    /// Base fee per commitment in base units.
    pub base_fee: Amount,
    /// Fee multiplier (scaled by 1000 — 1000 = 1.0x).
    pub fee_multiplier: u64,
    /// Total fees collected in the current epoch.
    pub epoch_fees: Amount,
}

/// The current set of validators.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ValidatorSet {
    /// Active validators ordered by stake (descending).
    pub validators: Vec<Validator>,
    /// Total stake across all active validators.
    pub total_stake: Amount,
    /// Current epoch number.
    pub epoch: u64,
}

impl ValidatorSet {
    /// Create an empty validator set.
    pub fn new(epoch: u64) -> Self {
        Self {
            validators: Vec::new(),
            total_stake: 0,
            epoch,
        }
    }

    /// Number of validators.
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// The maximum number of Byzantine faults tolerable: floor((n-1)/3).
    pub fn max_faults(&self) -> usize {
        if self.validators.is_empty() {
            0
        } else {
            (self.validators.len() - 1) / 3
        }
    }

    /// The quorum size: 2f+1.
    pub fn quorum_size(&self) -> usize {
        2 * self.max_faults() + 1
    }

    /// Check if a public key is in the validator set.
    pub fn contains(&self, pubkey: &PublicKey) -> bool {
        self.validators.iter().any(|v| v.pubkey == *pubkey)
    }

    /// Get validator by public key.
    pub fn get(&self, pubkey: &PublicKey) -> Option<&Validator> {
        self.validators.iter().find(|v| v.pubkey == *pubkey)
    }
}

/// A staking operation.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum StakeOperation {
    /// Stake tokens to become or increase stake as a validator.
    Stake {
        /// Validator public key.
        pubkey: PublicKey,
        /// Amount to stake.
        amount: Amount,
        /// Timestamp.
        timestamp: Timestamp,
        /// Signature by the staker.
        #[serde(with = "crate::primitives::serde_sig")]
        signature: Signature,
    },
    /// Unstake tokens (subject to bonding period).
    Unstake {
        /// Validator public key.
        pubkey: PublicKey,
        /// Amount to unstake.
        amount: Amount,
        /// Timestamp.
        timestamp: Timestamp,
        /// Signature by the staker.
        #[serde(with = "crate::primitives::serde_sig")]
        signature: Signature,
    },
}
