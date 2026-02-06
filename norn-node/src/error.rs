use thiserror::Error;

/// Errors that can occur in the node.
#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names, dead_code)]
pub enum NodeError {
    #[error("config error: {reason}")]
    ConfigError { reason: String },

    #[error("genesis error: {reason}")]
    GenesisError { reason: String },

    #[error("storage error: {0}")]
    StorageError(#[from] norn_storage::error::StorageError),

    #[error("relay error: {0}")]
    RelayError(String),

    #[error("weave error: {0}")]
    WeaveError(String),

    #[error("rpc error: {reason}")]
    RpcError { reason: String },

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = NodeError::ConfigError {
            reason: "missing field".to_string(),
        };
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_genesis_error_display() {
        let err = NodeError::GenesisError {
            reason: "invalid config".to_string(),
        };
        assert!(err.to_string().contains("invalid config"));
    }

    #[test]
    fn test_relay_error_display() {
        let err = NodeError::RelayError("connection failed".to_string());
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_io_error_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let node_err: NodeError = io_err.into();
        assert!(matches!(node_err, NodeError::IoError(_)));
    }
}
