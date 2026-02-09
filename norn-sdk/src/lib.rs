//! SDK for writing Norn Protocol loom smart contracts.
//!
//! # SDK v3 — Storage, Response Builder, Guards, Testing
//!
//! The recommended way to write contracts is with the [`Contract`] trait,
//! [`norn_entry!`] macro, typed [`storage`] primitives, and [`Response`]
//! builder:
//!
//! ```ignore
//! use norn_sdk::prelude::*;
//!
//! const OWNER: Item<Address> = Item::new("owner");
//! const BALANCES: Map<Address, u128> = Map::new("bal");
//!
//! #[derive(BorshSerialize, BorshDeserialize)]
//! pub struct MyToken;
//!
//! impl Contract for MyToken {
//!     type Exec = Exec;
//!     type Query = Query;
//!     fn init(ctx: &Context) -> Self {
//!         OWNER.save(&ctx.sender()).unwrap();
//!         MyToken
//!     }
//!     fn execute(&mut self, ctx: &Context, msg: Exec) -> ContractResult {
//!         // ...
//!         Ok(Response::new().add_attribute("action", "transfer"))
//!     }
//!     fn query(&self, _ctx: &Context, msg: Query) -> ContractResult {
//!         ok(BALANCES.load_or(&[0u8; 20], 0u128))
//!     }
//! }
//!
//! norn_entry!(MyToken);
//! ```
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

#[cfg(not(target_arch = "wasm32"))]
pub mod testing;

// Re-export key types at crate root for convenience.
pub use contract::{Context, Contract};
pub use error::ContractError;
pub use response::ContractResult;

// Re-export dlmalloc for the norn_entry! macro (wasm32 only).
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub use dlmalloc;
