use thiserror::Error;

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Key not found: {key}")]
    NotFound { key: String },

    #[error("Write error: {reason}")]
    WriteError { reason: String },

    #[error("Read error: {reason}")]
    ReadError { reason: String },

    #[error("SQLite error: {reason}")]
    SqliteError { reason: String },

    #[error("RocksDB error: {reason}")]
    RocksDbError { reason: String },

    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("Deserialization error: {reason}")]
    DeserializationError { reason: String },

    #[error("Batch error: {reason}")]
    BatchError { reason: String },
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        StorageError::SqliteError {
            reason: err.to_string(),
        }
    }
}

impl From<rocksdb::Error> for StorageError {
    fn from(err: rocksdb::Error) -> Self {
        StorageError::RocksDbError {
            reason: err.into_string(),
        }
    }
}
