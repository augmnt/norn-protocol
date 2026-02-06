use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::knot::Knot;
use crate::primitives::*;
use crate::thread::ThreadHeader;

/// Fraud proof variants that can be submitted to the weave.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum FraudProof {
    /// Two knots with the same version for the same thread.
    DoubleKnot {
        /// The thread that double-signed.
        thread_id: ThreadId,
        /// First knot at the disputed version.
        knot_a: Box<Knot>,
        /// Second knot at the same version.
        knot_b: Box<Knot>,
    },

    /// A commitment references a state that is stale or skips knots.
    StaleCommit {
        /// The thread with the stale commitment.
        thread_id: ThreadId,
        /// The stale commitment header.
        commitment: Box<ThreadHeader>,
        /// The knot(s) that should have been included.
        missing_knots: Vec<Knot>,
    },

    /// A loom state transition that violates the loom's rules.
    InvalidLoomTransition {
        /// The loom with the invalid transition.
        loom_id: LoomId,
        /// The knot containing the invalid transition.
        knot: Box<Knot>,
        /// Description of the rule violation.
        reason: String,
    },
}

/// Metadata about a submitted fraud proof.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct FraudProofSubmission {
    /// The fraud proof itself.
    pub proof: FraudProof,
    /// Who submitted the fraud proof.
    pub submitter: PublicKey,
    /// Timestamp of submission.
    pub timestamp: Timestamp,
    /// Signature by the submitter.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}
