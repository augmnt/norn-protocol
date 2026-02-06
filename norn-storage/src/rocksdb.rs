use rocksdb::{
    ColumnFamilyDescriptor, DBWithThreadMode, MultiThreaded, Options, WriteBatchWithTransaction,
};

use crate::error::StorageError;
use crate::traits::{BatchOp, BatchWriter, KvPairs, KvStore};

/// Default column families for the Norn storage.
pub const DEFAULT_CF: &str = "default";
pub const MERKLE_CF: &str = "merkle";
pub const BLOCKS_CF: &str = "blocks";
pub const COMMITMENTS_CF: &str = "commitments";

/// RocksDB-backed key-value store with column family support.
pub struct RocksDbStore {
    db: DBWithThreadMode<MultiThreaded>,
}

impl RocksDbStore {
    /// Open a RocksDB store at the given path with the specified column families.
    /// If `cf_names` is None, uses the default set of column families.
    pub fn new(path: &str, cf_names: Option<&[&str]>) -> Result<Self, StorageError> {
        let cfs = cf_names.unwrap_or(&[DEFAULT_CF, MERKLE_CF, BLOCKS_CF, COMMITMENTS_CF]);

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cfs
            .iter()
            .map(|name| {
                let cf_opts = Options::default();
                ColumnFamilyDescriptor::new(*name, cf_opts)
            })
            .collect();

        let db =
            DBWithThreadMode::<MultiThreaded>::open_cf_descriptors(&opts, path, cf_descriptors)?;

        Ok(Self { db })
    }

    /// Get a value from a specific column family.
    pub fn get_cf(&self, cf_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| StorageError::ReadError {
                reason: format!("Column family '{}' not found", cf_name),
            })?;
        let result = self.db.get_cf(&cf, key)?;
        Ok(result)
    }

    /// Put a value into a specific column family.
    pub fn put_cf(&self, cf_name: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| StorageError::WriteError {
                reason: format!("Column family '{}' not found", cf_name),
            })?;
        self.db.put_cf(&cf, key, value)?;
        Ok(())
    }

    /// Delete a key from a specific column family.
    pub fn delete_cf(&self, cf_name: &str, key: &[u8]) -> Result<(), StorageError> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| StorageError::WriteError {
                reason: format!("Column family '{}' not found", cf_name),
            })?;
        self.db.delete_cf(&cf, key)?;
        Ok(())
    }

    /// Prefix scan on a specific column family.
    pub fn prefix_scan_cf(&self, cf_name: &str, prefix: &[u8]) -> Result<KvPairs, StorageError> {
        let cf = self
            .db
            .cf_handle(cf_name)
            .ok_or_else(|| StorageError::ReadError {
                reason: format!("Column family '{}' not found", cf_name),
            })?;
        let iter = self.db.prefix_iterator_cf(&cf, prefix);
        let mut results = Vec::new();
        for item in iter {
            let (key, value) = item.map_err(|e| StorageError::ReadError {
                reason: e.to_string(),
            })?;
            if !key.starts_with(prefix) {
                break;
            }
            results.push((key.to_vec(), value.to_vec()));
        }
        Ok(results)
    }
}

impl KvStore for RocksDbStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let result = self.db.get(key)?;
        Ok(result)
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.db.put(key, value)?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        self.db.delete(key)?;
        Ok(())
    }

    fn exists(&self, key: &[u8]) -> Result<bool, StorageError> {
        let result = self.db.get(key)?;
        Ok(result.is_some())
    }

    fn prefix_scan(&self, prefix: &[u8]) -> Result<KvPairs, StorageError> {
        let iter = self.db.prefix_iterator(prefix);
        let mut results = Vec::new();
        for item in iter {
            let (key, value) = item.map_err(|e| StorageError::ReadError {
                reason: e.to_string(),
            })?;
            if !key.starts_with(prefix) {
                break;
            }
            results.push((key.to_vec(), value.to_vec()));
        }
        Ok(results)
    }
}

impl BatchWriter for RocksDbStore {
    fn write_batch(&self, ops: Vec<BatchOp>) -> Result<(), StorageError> {
        let mut batch = WriteBatchWithTransaction::<false>::default();
        for op in ops {
            match op {
                BatchOp::Put { key, value } => {
                    batch.put(&key, &value);
                }
                BatchOp::Delete { key } => {
                    batch.delete(&key);
                }
            }
        }
        self.db.write(batch)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("norn_rocksdb_test_{}", ts));
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_basic_crud() {
        let path = temp_dir();
        let store = RocksDbStore::new(&path, None).unwrap();
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

        drop(store);
        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_overwrite() {
        let path = temp_dir();
        let store = RocksDbStore::new(&path, None).unwrap();
        let key = b"key";
        store.put(key, b"value1").unwrap();
        store.put(key, b"value2").unwrap();
        assert_eq!(store.get(key).unwrap(), Some(b"value2".to_vec()));

        drop(store);
        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_prefix_scan() {
        let path = temp_dir();
        let store = RocksDbStore::new(&path, None).unwrap();
        store.put(b"prefix:a", b"1").unwrap();
        store.put(b"prefix:b", b"2").unwrap();
        store.put(b"prefix:c", b"3").unwrap();
        store.put(b"other:d", b"4").unwrap();

        let results = store.prefix_scan(b"prefix:").unwrap();
        assert_eq!(results.len(), 3);

        drop(store);
        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_batch_put_and_delete() {
        let path = temp_dir();
        let store = RocksDbStore::new(&path, None).unwrap();
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

        drop(store);
        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_column_family_ops() {
        let path = temp_dir();
        let store = RocksDbStore::new(&path, None).unwrap();

        // Put in merkle CF
        store.put_cf(MERKLE_CF, b"mk_key", b"mk_value").unwrap();
        assert_eq!(
            store.get_cf(MERKLE_CF, b"mk_key").unwrap(),
            Some(b"mk_value".to_vec())
        );

        // Should not be visible in default CF
        assert_eq!(store.get(b"mk_key").unwrap(), None);

        // Delete from CF
        store.delete_cf(MERKLE_CF, b"mk_key").unwrap();
        assert_eq!(store.get_cf(MERKLE_CF, b"mk_key").unwrap(), None);

        drop(store);
        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_get_nonexistent() {
        let path = temp_dir();
        let store = RocksDbStore::new(&path, None).unwrap();
        assert_eq!(store.get(b"no_such_key").unwrap(), None);

        drop(store);
        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }
}
