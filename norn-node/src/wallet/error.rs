use thiserror::Error;

/// Errors that can occur in wallet operations.
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("no active wallet set — run `norn-node wallet use <name>` to select one")]
    NoActiveWallet,

    #[error("wallet '{0}' not found")]
    WalletNotFound(String),

    #[error("wallet '{0}' already exists")]
    WalletAlreadyExists(String),

    #[error("invalid password: decryption failed")]
    InvalidPassword,

    #[error("invalid address: {0}")]
    InvalidAddress(String),

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("insufficient balance: have {available}, need {required}")]
    InsufficientBalance { available: String, required: String },

    #[error("rpc error: {0}")]
    RpcError(String),

    #[error("crypto error: {0}")]
    CryptoError(#[from] norn_types::error::NornError),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("config error: {0}")]
    ConfigError(String),

    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for WalletError {
    fn from(e: serde_json::Error) -> Self {
        WalletError::SerializationError(e.to_string())
    }
}
