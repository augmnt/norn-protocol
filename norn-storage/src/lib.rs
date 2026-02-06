//! Storage abstraction for the Norn Protocol.
//!
//! Provides a [`KvStore`](traits::KvStore) trait with memory, SQLite, and RocksDB
//! backends, plus specialized stores for Merkle trees, Threads, and Weave state.

pub mod error;
pub mod memory;
pub mod merkle_store;
pub mod rocksdb;
pub mod sqlite;
pub mod thread_store;
pub mod traits;
pub mod weave_store;
