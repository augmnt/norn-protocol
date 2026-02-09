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
