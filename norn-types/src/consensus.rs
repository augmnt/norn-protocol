use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::primitives::*;

/// A vote from a validator during consensus.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Vote {
    /// The view number this vote is for.
    pub view: u64,
    /// The block hash being voted on.
    pub block_hash: Hash,
    /// The voter's public key.
    pub voter: PublicKey,
    /// Signature over (view, block_hash).
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A quorum certificate — 2f+1 votes for a block at a given phase.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct QuorumCertificate {
    /// The view number.
    pub view: u64,
    /// The block hash.
    pub block_hash: Hash,
    /// The phase this QC certifies.
    pub phase: ConsensusPhase,
    /// The votes forming the quorum.
    pub votes: Vec<Vote>,
}

/// Consensus phases in 3-phase HotStuff.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
pub enum ConsensusPhase {
    /// First phase: prepare.
    Prepare,
    /// Second phase: pre-commit.
    PreCommit,
    /// Third phase: commit.
    Commit,
}

/// Proof for a view change (timeout certificate).
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ViewChangeProof {
    /// The view being changed from.
    pub old_view: u64,
    /// The new view.
    pub new_view: u64,
    /// Timeout votes from 2f+1 validators.
    pub timeout_votes: Vec<TimeoutVote>,
    /// The highest QC known by any voter.
    pub highest_qc: Option<QuorumCertificate>,
}

/// A timeout vote from a validator requesting view change.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct TimeoutVote {
    /// The view that timed out.
    pub view: u64,
    /// The voter's public key.
    pub voter: PublicKey,
    /// The highest QC this voter knows about.
    pub highest_qc_view: u64,
    /// Signature over (view, highest_qc_view).
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// Messages exchanged during consensus.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Proposal from the leader with a block and its justification.
    Prepare {
        /// The view number for this proposal.
        view: u64,
        /// The proposed block hash.
        block_hash: Hash,
        /// The block data (serialized WeaveBlock).
        block_data: Vec<u8>,
        /// QC from the previous phase justifying this proposal.
        justify: Option<QuorumCertificate>,
    },

    /// Vote for the prepare phase.
    PrepareVote(Vote),

    /// Pre-commit message from leader with prepare QC.
    PreCommit {
        /// The view number.
        view: u64,
        /// The prepare QC.
        prepare_qc: QuorumCertificate,
    },

    /// Vote for the pre-commit phase.
    PreCommitVote(Vote),

    /// Commit message from leader with pre-commit QC.
    Commit {
        /// The view number.
        view: u64,
        /// The pre-commit QC.
        precommit_qc: QuorumCertificate,
    },

    /// Vote for the commit phase.
    CommitVote(Vote),

    /// Request to change view (timeout).
    ViewChange(TimeoutVote),

    /// New view message from the new leader.
    NewView {
        /// The new view number.
        view: u64,
        /// The view change proof.
        proof: ViewChangeProof,
    },
}
