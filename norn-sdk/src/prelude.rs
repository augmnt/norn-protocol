//! One-stop import for loom contract developers.
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//! ```

// SDK v2 — core types
pub use crate::contract::{Context, Contract};
pub use crate::error::ContractError;
pub use crate::response::{ok, ok_bytes, ok_empty, Attribute, ContractResult, Response};
pub use crate::types::{Address, TokenId};

// SDK v3 — storage, guards, address helpers
pub use crate::addr::{addr_to_hex, hex_to_addr, ZERO_ADDRESS};
pub use crate::storage::{Item, Map, StorageKey};

// Guard macros (exported at crate root by #[macro_export])
#[doc(hidden)]
pub use crate::ensure;
#[doc(hidden)]
pub use crate::ensure_eq;
#[doc(hidden)]
pub use crate::ensure_ne;

// borsh derives
pub use borsh::{BorshDeserialize, BorshSerialize};

// alloc essentials
pub use alloc::{format, string::String, vec, vec::Vec};

// Re-export the norn_entry! macro so `use norn_sdk::prelude::*` brings it into scope.
#[doc(hidden)]
pub use crate::norn_entry;
