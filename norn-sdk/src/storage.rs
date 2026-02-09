//! Typed storage primitives for loom contracts.
//!
//! [`Item<T>`] stores a single value under a fixed key. [`Map<K, V>`] stores
//! values keyed by an arbitrary key. Both are `const fn` constructable so they
//! can be declared as module-level constants:
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//!
//! const OWNER: Item<Address> = Item::new("owner");
//! const BALANCES: Map<Address, u128> = Map::new("bal");
//! ```
//!
//! Storage primitives call [`host::state_get`](crate::host::state_get) /
//! [`host::state_set`](crate::host::state_set) directly, so they work both
//! on wasm32 (real host) and in native tests (thread-local mock).

use alloc::vec::Vec;
use core::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::ContractError;
use crate::host;

// ═══════════════════════════════════════════════════════════════════════════
// StorageKey trait
// ═══════════════════════════════════════════════════════════════════════════

/// Types that can be used as keys in [`Map`] storage.
pub trait StorageKey {
    /// Serialize the key to bytes for storage.
    fn storage_key(&self) -> Vec<u8>;
}

impl<const N: usize> StorageKey for [u8; N] {
    fn storage_key(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl StorageKey for &[u8] {
    fn storage_key(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl StorageKey for u64 {
    fn storage_key(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl StorageKey for u128 {
    fn storage_key(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl StorageKey for &str {
    fn storage_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl StorageKey for alloc::string::String {
    fn storage_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Item<T> — single-value storage
// ═══════════════════════════════════════════════════════════════════════════

/// Stores a single borsh-serialized value under a fixed namespace key.
///
/// ```ignore
/// const OWNER: Item<Address> = Item::new("owner");
///
/// OWNER.save(&ctx.sender())?;
/// let owner = OWNER.load()?;
/// ```
pub struct Item<T> {
    namespace: &'static str,
    _marker: PhantomData<T>,
}

impl<T> Item<T> {
    /// Create a new `Item` with the given namespace.
    pub const fn new(namespace: &'static str) -> Self {
        Item {
            namespace,
            _marker: PhantomData,
        }
    }
}

impl<T: BorshSerialize + BorshDeserialize> Item<T> {
    /// Save a value to storage.
    pub fn save(&self, value: &T) -> Result<(), ContractError> {
        let bytes = borsh::to_vec(value)
            .map_err(|e| ContractError::Custom(alloc::format!("serialize: {e}")))?;
        host::state_set(self.namespace.as_bytes(), &bytes);
        Ok(())
    }

    /// Load the value from storage, returning `NotFound` if absent.
    pub fn load(&self) -> Result<T, ContractError> {
        match host::state_get(self.namespace.as_bytes()) {
            Some(bytes) if !bytes.is_empty() => BorshDeserialize::try_from_slice(&bytes)
                .map_err(|e| ContractError::Custom(alloc::format!("deserialize: {e}"))),
            _ => Err(ContractError::NotFound(alloc::format!(
                "item '{}' not found",
                self.namespace
            ))),
        }
    }

    /// Load the value, returning `default` if absent.
    pub fn load_or(&self, default: T) -> T {
        self.load().unwrap_or(default)
    }

    /// Load the value, returning `T::default()` if absent.
    pub fn load_or_default(&self) -> T
    where
        T: Default,
    {
        self.load().unwrap_or_default()
    }

    /// Check if the item exists in storage.
    pub fn exists(&self) -> bool {
        matches!(host::state_get(self.namespace.as_bytes()), Some(b) if !b.is_empty())
    }

    /// Remove the item from storage.
    pub fn remove(&self) {
        host::state_remove(self.namespace.as_bytes());
    }

    /// Load, apply a function, save, and return the updated value.
    pub fn update<F>(&self, f: F) -> Result<T, ContractError>
    where
        F: FnOnce(T) -> Result<T, ContractError>,
    {
        let value = self.load()?;
        let updated = f(value)?;
        self.save(&updated)?;
        Ok(updated)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Map<K, V> — keyed storage
// ═══════════════════════════════════════════════════════════════════════════

/// Stores borsh-serialized values keyed by an arbitrary [`StorageKey`].
///
/// The full storage key is `namespace_bytes + 0x00 + key_bytes`.
///
/// ```ignore
/// const BALANCES: Map<Address, u128> = Map::new("bal");
///
/// BALANCES.save(&addr, &1000u128)?;
/// let balance = BALANCES.load_or(&addr, 0u128);
/// ```
pub struct Map<K, V> {
    namespace: &'static str,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> Map<K, V> {
    /// Create a new `Map` with the given namespace.
    pub const fn new(namespace: &'static str) -> Self {
        Map {
            namespace,
            _marker: PhantomData,
        }
    }
}

impl<K: StorageKey, V: BorshSerialize + BorshDeserialize> Map<K, V> {
    fn full_key(&self, key: &K) -> Vec<u8> {
        let ns = self.namespace.as_bytes();
        let k = key.storage_key();
        let mut full = Vec::with_capacity(ns.len() + 1 + k.len());
        full.extend_from_slice(ns);
        full.push(0x00); // separator
        full.extend_from_slice(&k);
        full
    }

    /// Save a value at the given key.
    pub fn save(&self, key: &K, value: &V) -> Result<(), ContractError> {
        let bytes = borsh::to_vec(value)
            .map_err(|e| ContractError::Custom(alloc::format!("serialize: {e}")))?;
        host::state_set(&self.full_key(key), &bytes);
        Ok(())
    }

    /// Load the value at the given key, returning `NotFound` if absent.
    pub fn load(&self, key: &K) -> Result<V, ContractError> {
        match host::state_get(&self.full_key(key)) {
            Some(bytes) if !bytes.is_empty() => BorshDeserialize::try_from_slice(&bytes)
                .map_err(|e| ContractError::Custom(alloc::format!("deserialize: {e}"))),
            _ => Err(ContractError::NotFound(alloc::format!(
                "map '{}' key not found",
                self.namespace
            ))),
        }
    }

    /// Load the value at the given key, returning `default` if absent.
    pub fn load_or(&self, key: &K, default: V) -> V {
        self.load(key).unwrap_or(default)
    }

    /// Load the value at the given key, returning `V::default()` if absent.
    pub fn load_or_default(&self, key: &K) -> V
    where
        V: Default,
    {
        self.load(key).unwrap_or_default()
    }

    /// Check if a key exists in the map.
    pub fn has(&self, key: &K) -> bool {
        matches!(host::state_get(&self.full_key(key)), Some(b) if !b.is_empty())
    }

    /// Remove a key from the map.
    pub fn remove(&self, key: &K) {
        host::state_remove(&self.full_key(key));
    }

    /// Load, apply a function, save, and return the updated value.
    pub fn update<F>(&self, key: &K, f: F) -> Result<V, ContractError>
    where
        F: FnOnce(V) -> Result<V, ContractError>,
    {
        let value = self.load(key)?;
        let updated = f(value)?;
        self.save(key, &updated)?;
        Ok(updated)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IndexedMap<K, V> — keyed storage with iteration
// ═══════════════════════════════════════════════════════════════════════════

/// Like [`Map`], but maintains a client-side index so keys can be enumerated.
///
/// This uses 3 extra storage writes per insert (key-at-index, reverse-index,
/// count) and swap-and-pop for O(1) removal. Use `Map` when iteration is not
/// needed; use `IndexedMap` when you need `keys()` or `range()`.
///
/// Storage layout:
/// - `{ns}\x00{key_bytes}` → borsh(V) (same as Map)
/// - `{ns}\x01idx\x00{index_le_u64}` → borsh(K)
/// - `{ns}\x01count` → borsh(u64)
/// - `{ns}\x01rev\x00{key_bytes}` → borsh(u64)
///
/// ```ignore
/// const HOLDERS: IndexedMap<Address, u128> = IndexedMap::new("holders");
///
/// HOLDERS.save(&addr, &1000u128)?;
/// let all_keys = HOLDERS.keys();
/// let page = HOLDERS.range(0, 10);
/// ```
pub struct IndexedMap<K, V> {
    namespace: &'static str,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> IndexedMap<K, V> {
    /// Create a new `IndexedMap` with the given namespace.
    pub const fn new(namespace: &'static str) -> Self {
        IndexedMap {
            namespace,
            _marker: PhantomData,
        }
    }
}

impl<K: StorageKey + BorshSerialize + BorshDeserialize, V: BorshSerialize + BorshDeserialize>
    IndexedMap<K, V>
{
    // ── Internal key builders ──────────────────────────────────────────

    /// Value key: `{ns}\x00{key_bytes}`
    fn value_key(&self, key: &K) -> Vec<u8> {
        let ns = self.namespace.as_bytes();
        let k = key.storage_key();
        let mut full = Vec::with_capacity(ns.len() + 1 + k.len());
        full.extend_from_slice(ns);
        full.push(0x00);
        full.extend_from_slice(&k);
        full
    }

    /// Index → key: `{ns}\x01idx\x00{index_le_u64}`
    fn idx_key(&self, index: u64) -> Vec<u8> {
        let ns = self.namespace.as_bytes();
        let idx_bytes = index.to_le_bytes();
        let mut full = Vec::with_capacity(ns.len() + 5 + 8);
        full.extend_from_slice(ns);
        full.extend_from_slice(b"\x01idx\x00");
        full.extend_from_slice(&idx_bytes);
        full
    }

    /// Count key: `{ns}\x01count`
    fn count_key(&self) -> Vec<u8> {
        let ns = self.namespace.as_bytes();
        let mut full = Vec::with_capacity(ns.len() + 6);
        full.extend_from_slice(ns);
        full.extend_from_slice(b"\x01count");
        full
    }

    /// Reverse index (key → index): `{ns}\x01rev\x00{key_bytes}`
    fn rev_key(&self, key: &K) -> Vec<u8> {
        let ns = self.namespace.as_bytes();
        let k = key.storage_key();
        let mut full = Vec::with_capacity(ns.len() + 5 + k.len());
        full.extend_from_slice(ns);
        full.extend_from_slice(b"\x01rev\x00");
        full.extend_from_slice(&k);
        full
    }

    // ── Internal helpers ───────────────────────────────────────────────

    fn read_count(&self) -> u64 {
        match host::state_get(&self.count_key()) {
            Some(bytes) if !bytes.is_empty() => u64::try_from_slice(&bytes).unwrap_or(0),
            _ => 0,
        }
    }

    fn write_count(&self, count: u64) {
        let bytes = borsh::to_vec(&count).unwrap_or_default();
        host::state_set(&self.count_key(), &bytes);
    }

    fn read_key_at(&self, index: u64) -> Option<K> {
        match host::state_get(&self.idx_key(index)) {
            Some(bytes) if !bytes.is_empty() => K::try_from_slice(&bytes).ok(),
            _ => None,
        }
    }

    fn write_key_at(&self, index: u64, key: &K) {
        let bytes = borsh::to_vec(key).unwrap_or_default();
        host::state_set(&self.idx_key(index), &bytes);
    }

    fn read_rev(&self, key: &K) -> Option<u64> {
        match host::state_get(&self.rev_key(key)) {
            Some(bytes) if !bytes.is_empty() => u64::try_from_slice(&bytes).ok(),
            _ => None,
        }
    }

    fn write_rev(&self, key: &K, index: u64) {
        let bytes = borsh::to_vec(&index).unwrap_or_default();
        host::state_set(&self.rev_key(key), &bytes);
    }

    fn remove_rev(&self, key: &K) {
        host::state_remove(&self.rev_key(key));
    }

    fn remove_idx(&self, index: u64) {
        host::state_remove(&self.idx_key(index));
    }

    // ── Public API ─────────────────────────────────────────────────────

    /// Save a value at the given key. If the key is new, adds it to the index.
    pub fn save(&self, key: &K, value: &V) -> Result<(), ContractError> {
        let bytes = borsh::to_vec(value)
            .map_err(|e| ContractError::Custom(alloc::format!("serialize: {e}")))?;
        host::state_set(&self.value_key(key), &bytes);

        // Only add to index if key is not already tracked.
        if self.read_rev(key).is_none() {
            let count = self.read_count();
            self.write_key_at(count, key);
            self.write_rev(key, count);
            self.write_count(count + 1);
        }
        Ok(())
    }

    /// Load the value at the given key, returning `NotFound` if absent.
    pub fn load(&self, key: &K) -> Result<V, ContractError> {
        match host::state_get(&self.value_key(key)) {
            Some(bytes) if !bytes.is_empty() => BorshDeserialize::try_from_slice(&bytes)
                .map_err(|e| ContractError::Custom(alloc::format!("deserialize: {e}"))),
            _ => Err(ContractError::NotFound(alloc::format!(
                "indexed_map '{}' key not found",
                self.namespace
            ))),
        }
    }

    /// Load the value at the given key, returning `default` if absent.
    pub fn load_or(&self, key: &K, default: V) -> V {
        self.load(key).unwrap_or(default)
    }

    /// Check if a key exists.
    pub fn has(&self, key: &K) -> bool {
        self.read_rev(key).is_some()
    }

    /// Remove a key and its value. Uses swap-and-pop for O(1) removal.
    pub fn remove(&self, key: &K) -> Result<(), ContractError> {
        let index = match self.read_rev(key) {
            Some(i) => i,
            None => return Ok(()), // not present, no-op
        };

        let count = self.read_count();
        let last_index = count - 1;

        // If not the last entry, swap with last.
        if index != last_index {
            if let Some(last_key) = self.read_key_at(last_index) {
                self.write_key_at(index, &last_key);
                self.write_rev(&last_key, index);
            }
        }

        // Remove the (now-last) index entry and reverse mapping.
        self.remove_idx(last_index);
        self.remove_rev(key);

        // Remove the value.
        host::state_remove(&self.value_key(key));

        // Decrement count.
        self.write_count(last_index);
        Ok(())
    }

    /// Return the number of entries.
    pub fn len(&self) -> u64 {
        self.read_count()
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.read_count() == 0
    }

    /// Return all keys. For large maps, prefer `range()`.
    pub fn keys(&self) -> Vec<K> {
        let count = self.read_count();
        let mut keys = Vec::with_capacity(count as usize);
        for i in 0..count {
            if let Some(key) = self.read_key_at(i) {
                keys.push(key);
            }
        }
        keys
    }

    /// Return a paginated slice of (key, value) pairs.
    ///
    /// `start` is the 0-based index, `end` is exclusive.
    pub fn range(&self, start: u64, end: u64) -> Vec<(K, V)> {
        let count = self.read_count();
        let end = end.min(count);
        let start = start.min(end);
        let mut results = Vec::with_capacity((end - start) as usize);
        for i in start..end {
            if let Some(key) = self.read_key_at(i) {
                if let Ok(value) = self.load(&key) {
                    results.push((key, value));
                }
            }
        }
        results
    }
}
