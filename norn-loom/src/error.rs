use thiserror::Error;

/// Errors that can occur in the loom subsystem.
#[derive(Debug, Error)]
pub enum LoomError {
    #[error("Runtime error: {reason}")]
    RuntimeError { reason: String },

    #[error("Gas exhausted: used {used} of {limit}")]
    GasExhausted { used: u64, limit: u64 },

    #[error("Invalid bytecode: {reason}")]
    InvalidBytecode { reason: String },

    #[error("State error: {reason}")]
    StateError { reason: String },

    #[error("Loom not found: {loom_id:?}")]
    LoomNotFound { loom_id: [u8; 32] },

    #[error("Not a participant: {address:?}")]
    NotParticipant { address: [u8; 20] },

    #[error("Participant limit exceeded: {count} > {max}")]
    ParticipantLimitExceeded { count: usize, max: usize },

    #[error("Invalid transition: {reason}")]
    InvalidTransition { reason: String },

    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("Host error: {reason}")]
    HostError { reason: String },

    #[error("Storage error: {0}")]
    StorageError(#[from] norn_storage::error::StorageError),
}
