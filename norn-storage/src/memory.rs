use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::error::StorageError;
use crate::traits::{BatchOp, BatchWriter, KvPairs, KvStore};

/// In-memory key-value store backed by a BTreeMap.
/// Uses BTreeMap so that prefix_scan can leverage ordered iteration.
pub struct MemoryStore {
    data: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryStore {
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self {
            data: RwLock::new(BTreeMap::new()),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KvStore for MemoryStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let data = self.data.read().map_err(|e| StorageError::ReadError {
            reason: e.to_string(),
        })?;
        Ok(data.get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let mut data = self.data.write().map_err(|e| StorageError::WriteError {
            reason: e.to_string(),
        })?;
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        let mut data = self.data.write().map_err(|e| StorageError::WriteError {
            reason: e.to_string(),
        })?;
        data.remove(key);
        Ok(())
    }

    fn exists(&self, key: &[u8]) -> Result<bool, StorageError> {
        let data = self.data.read().map_err(|e| StorageError::ReadError {
            reason: e.to_string(),
        })?;
        Ok(data.contains_key(key))
    }

    fn prefix_scan(&self, prefix: &[u8]) -> Result<KvPairs, StorageError> {
        let data = self.data.read().map_err(|e| StorageError::ReadError {
            reason: e.to_string(),
        })?;
        let results: KvPairs = data
            .range(prefix.to_vec()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Ok(results)
    }
}

impl BatchWriter for MemoryStore {
    fn write_batch(&self, ops: Vec<BatchOp>) -> Result<(), StorageError> {
        let mut data = self.data.write().map_err(|e| StorageError::BatchError {
            reason: e.to_string(),
        })?;
        for op in ops {
            match op {
                BatchOp::Put { key, value } => {
                    data.insert(key, value);
                }
                BatchOp::Delete { key } => {
                    data.remove(&key);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_crud() {
        let store = MemoryStore::new();
        let key = b"test_key";
        let value = b"test_value";

        // Put and get
        store.put(key, value).unwrap();
        assert_eq!(store.get(key).unwrap(), Some(value.to_vec()));

        // Exists
        assert!(store.exists(key).unwrap());
        assert!(!store.exists(b"nonexistent").unwrap());

        // Delete
        store.delete(key).unwrap();
        assert_eq!(store.get(key).unwrap(), None);
        assert!(!store.exists(key).unwrap());
    }

    #[test]
    fn test_overwrite() {
        let store = MemoryStore::new();
        let key = b"key";
        store.put(key, b"value1").unwrap();
        store.put(key, b"value2").unwrap();
        assert_eq!(store.get(key).unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_prefix_scan() {
        let store = MemoryStore::new();
        store.put(b"prefix:a", b"1").unwrap();
        store.put(b"prefix:b", b"2").unwrap();
        store.put(b"prefix:c", b"3").unwrap();
        store.put(b"other:d", b"4").unwrap();

        let results = store.prefix_scan(b"prefix:").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, b"prefix:a".to_vec());
        assert_eq!(results[1].0, b"prefix:b".to_vec());
        assert_eq!(results[2].0, b"prefix:c".to_vec());
    }

    #[test]
    fn test_prefix_scan_empty() {
        let store = MemoryStore::new();
        let results = store.prefix_scan(b"nonexistent:").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_batch_put_and_delete() {
        let store = MemoryStore::new();
        store.put(b"to_delete", b"value").unwrap();

        let ops = vec![
            BatchOp::Put {
                key: b"batch_key1".to_vec(),
                value: b"batch_val1".to_vec(),
            },
            BatchOp::Put {
                key: b"batch_key2".to_vec(),
                value: b"batch_val2".to_vec(),
            },
            BatchOp::Delete {
                key: b"to_delete".to_vec(),
            },
        ];

        store.write_batch(ops).unwrap();

        assert_eq!(
            store.get(b"batch_key1").unwrap(),
            Some(b"batch_val1".to_vec())
        );
        assert_eq!(
            store.get(b"batch_key2").unwrap(),
            Some(b"batch_val2".to_vec())
        );
        assert_eq!(store.get(b"to_delete").unwrap(), None);
    }

    #[test]
    fn test_get_nonexistent() {
        let store = MemoryStore::new();
        assert_eq!(store.get(b"no_such_key").unwrap(), None);
    }

    #[test]
    fn test_delete_nonexistent() {
        let store = MemoryStore::new();
        // Deleting a non-existent key should not error.
        store.delete(b"no_such_key").unwrap();
    }
}
