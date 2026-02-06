use thiserror::Error;

/// Errors that can occur in the spindle service.
#[derive(Debug, Error)]
pub enum SpindleError {
    #[error("Monitor error: {reason}")]
    MonitorError { reason: String },

    #[error("Fraud detected on thread {thread_id}: {proof_type}")]
    FraudDetected {
        thread_id: String,
        proof_type: String,
    },

    #[error("Service error: {reason}")]
    ServiceError { reason: String },

    #[error("Rate limit exceeded for peer: {peer}")]
    RateLimitExceeded { peer: String },

    #[error("Storage error: {0}")]
    StorageError(#[from] norn_storage::error::StorageError),
}
