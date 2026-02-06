use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::error::StorageError;
use crate::traits::{BatchOp, BatchWriter, KvPairs, KvStore};

/// SQLite-backed key-value store.
/// Uses a single `kv` table with BLOB key and BLOB value columns.
pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    /// Create a new SQLite store at the given path.
    /// Use `:memory:` for an in-memory database (useful for tests).
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv (key BLOB PRIMARY KEY, value BLOB NOT NULL)",
            [],
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl KvStore for SqliteStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::ReadError {
            reason: e.to_string(),
        })?;
        let mut stmt = conn.prepare_cached("SELECT value FROM kv WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            Some(row) => {
                let value: Vec<u8> = row.get(0)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::WriteError {
            reason: e.to_string(),
        })?;
        conn.execute(
            "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::WriteError {
            reason: e.to_string(),
        })?;
        conn.execute("DELETE FROM kv WHERE key = ?1", params![key])?;
        Ok(())
    }

    fn exists(&self, key: &[u8]) -> Result<bool, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::ReadError {
            reason: e.to_string(),
        })?;
        let mut stmt = conn.prepare_cached("SELECT 1 FROM kv WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        Ok(rows.next()?.is_some())
    }

    fn prefix_scan(&self, prefix: &[u8]) -> Result<KvPairs, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::ReadError {
            reason: e.to_string(),
        })?;

        // Compute the upper bound for the prefix range.
        // Increment the last byte of the prefix; if it overflows, drop it and increment the
        // previous byte, etc. If all bytes overflow we just scan to the end.
        let upper_bound = increment_prefix(prefix);

        let mut results = Vec::new();
        match upper_bound {
            Some(ref ub) => {
                let mut stmt = conn.prepare_cached(
                    "SELECT key, value FROM kv WHERE key >= ?1 AND key < ?2 ORDER BY key",
                )?;
                let mut rows = stmt.query(params![prefix, ub])?;
                while let Some(row) = rows.next()? {
                    let k: Vec<u8> = row.get(0)?;
                    let v: Vec<u8> = row.get(1)?;
                    results.push((k, v));
                }
            }
            None => {
                let mut stmt =
                    conn.prepare_cached("SELECT key, value FROM kv WHERE key >= ?1 ORDER BY key")?;
                let mut rows = stmt.query(params![prefix])?;
                while let Some(row) = rows.next()? {
                    let k: Vec<u8> = row.get(0)?;
                    if !k.starts_with(prefix) {
                        break;
                    }
                    let v: Vec<u8> = row.get(1)?;
                    results.push((k, v));
                }
            }
        }

        Ok(results)
    }
}

impl BatchWriter for SqliteStore {
    fn write_batch(&self, ops: Vec<BatchOp>) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::BatchError {
            reason: e.to_string(),
        })?;
        let tx = conn.unchecked_transaction()?;
        for op in ops {
            match op {
                BatchOp::Put { key, value } => {
                    tx.execute(
                        "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
                        params![key, value],
                    )?;
                }
                BatchOp::Delete { key } => {
                    tx.execute("DELETE FROM kv WHERE key = ?1", params![key])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }
}

/// Increment a byte prefix to compute an exclusive upper bound.
/// Returns None if the prefix is all 0xFF bytes (no upper bound).
fn increment_prefix(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut result = prefix.to_vec();
    for i in (0..result.len()).rev() {
        if result[i] < 0xFF {
            result[i] += 1;
            result.truncate(i + 1);
            return Some(result);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> SqliteStore {
        SqliteStore::new(":memory:").unwrap()
    }

    #[test]
    fn test_basic_crud() {
        let store = make_store();
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
        let store = make_store();
        let key = b"key";
        store.put(key, b"value1").unwrap();
        store.put(key, b"value2").unwrap();
        assert_eq!(store.get(key).unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_prefix_scan() {
        let store = make_store();
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
        let store = make_store();
        let results = store.prefix_scan(b"nonexistent:").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_batch_put_and_delete() {
        let store = make_store();
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
        let store = make_store();
        assert_eq!(store.get(b"no_such_key").unwrap(), None);
    }

    #[test]
    fn test_delete_nonexistent() {
        let store = make_store();
        store.delete(b"no_such_key").unwrap();
    }
}
