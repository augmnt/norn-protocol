use thiserror::Error;

/// Errors specific to the weave consensus layer.
#[derive(Debug, Error)]
pub enum WeaveError {
    #[error("invalid block: {reason}")]
    InvalidBlock { reason: String },

    #[error("invalid commitment: {reason}")]
    InvalidCommitment { reason: String },

    #[error("invalid registration: {reason}")]
    InvalidRegistration { reason: String },

    #[error("invalid fraud proof: {reason}")]
    InvalidFraudProof { reason: String },

    #[error("duplicate thread: {thread_id:?}")]
    DuplicateThread { thread_id: [u8; 20] },

    #[error("stale commitment: age {age}s exceeds max {max_age}s")]
    StaleCommitment { age: u64, max_age: u64 },

    #[error("consensus error: {reason}")]
    ConsensusError { reason: String },

    #[error("staking error: {reason}")]
    StakingError { reason: String },

    #[error("mempool full")]
    MempoolFull,

    #[error("not the current leader")]
    NotLeader,

    #[error("insufficient quorum: have {have}, need {need}")]
    InsufficientQuorum { have: usize, need: usize },

    #[error("view change required: current view {current_view}")]
    ViewChangeRequired { current_view: u64 },

    #[error("storage error: {0}")]
    StorageError(#[from] norn_storage::error::StorageError),
}
