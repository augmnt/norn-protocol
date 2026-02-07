use std::sync::Arc;

use crate::error::StorageError;

/// Result type for prefix scan operations: a list of key-value byte pairs.
pub type KvPairs = Vec<(Vec<u8>, Vec<u8>)>;

/// Batch operation for atomic writes.
#[derive(Debug, Clone)]
pub enum BatchOp {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}

/// Core key-value store trait.
pub trait KvStore: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    fn delete(&self, key: &[u8]) -> Result<(), StorageError>;
    fn exists(&self, key: &[u8]) -> Result<bool, StorageError>;
    fn prefix_scan(&self, prefix: &[u8]) -> Result<KvPairs, StorageError>;
}

/// Atomic batch writer trait.
pub trait BatchWriter: KvStore {
    fn write_batch(&self, ops: Vec<BatchOp>) -> Result<(), StorageError>;
}

/// Blanket implementation of KvStore for `Arc<S>` so that a store can be shared
/// across multiple owners (e.g. for persistence-across-restart tests).
impl<S: KvStore + ?Sized> KvStore for Arc<S> {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        (**self).get(key)
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        (**self).put(key, value)
    }

    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        (**self).delete(key)
    }

    fn exists(&self, key: &[u8]) -> Result<bool, StorageError> {
        (**self).exists(key)
    }

    fn prefix_scan(&self, prefix: &[u8]) -> Result<KvPairs, StorageError> {
        (**self).prefix_scan(prefix)
    }
}

/// Blanket implementation of KvStore for `Box<dyn KvStore>` so that a
/// type-erased store can be used wherever a concrete store is expected.
impl KvStore for Box<dyn KvStore> {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        (**self).get(key)
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        (**self).put(key, value)
    }

    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        (**self).delete(key)
    }

    fn exists(&self, key: &[u8]) -> Result<bool, StorageError> {
        (**self).exists(key)
    }

    fn prefix_scan(&self, prefix: &[u8]) -> Result<KvPairs, StorageError> {
        (**self).prefix_scan(prefix)
    }
}
