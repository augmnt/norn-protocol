//! SDK for writing Norn Protocol loom smart contracts.
//!
//! # SDK v5 — Proc-Macro DX
//!
//! The recommended way to write contracts uses `#[norn_contract]`:
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//!
//! #[norn_contract]
//! pub struct Counter { value: u64 }
//!
//! #[norn_contract]
//! impl Counter {
//!     #[init]
//!     pub fn new(_ctx: &Context) -> Self { Counter { value: 0 } }
//!
//!     #[execute]
//!     pub fn increment(&mut self, _ctx: &Context) -> ContractResult {
//!         self.value += 1;
//!         ok(self.value)
//!     }
//!
//!     #[query]
//!     pub fn get_value(&self, _ctx: &Context) -> ContractResult {
//!         ok(self.value)
//!     }
//! }
//! ```
//!
//! The macro generates borsh derives, dispatch enums, `Contract` trait impl,
//! and `norn_entry!` — so every line is business logic.
//!
//! The manual `Contract` trait + `norn_entry!` approach is still fully
//! supported for advanced use cases.
//!
//! # Low-level API
//!
//! The [`host`], [`output`], and [`encoding`] modules are still available for
//! advanced use cases that need direct access to host functions.

#![no_std]

extern crate alloc;

#[cfg(not(target_arch = "wasm32"))]
extern crate std;

// -- Low-level modules (backward compatible) --
pub mod encoding;
pub mod host;
pub mod output;

// -- SDK v2 modules --
pub mod contract;
pub mod entry;
pub mod error;
pub mod prelude;
pub mod response;
pub mod types;

// -- SDK v3 modules --
pub mod addr;
pub mod guard;
pub mod storage;

// -- SDK v3 standard library --
pub mod stdlib;

#[cfg(not(target_arch = "wasm32"))]
pub mod testing;

// Re-export key types at crate root for convenience.
pub use contract::{Context, Contract};
pub use error::ContractError;
pub use response::ContractResult;

// Re-export the proc macro from norn-sdk-macros.
pub use norn_sdk_macros::norn_contract;

// Re-export dlmalloc for the norn_entry! macro (wasm32 only).
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub use dlmalloc;
