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
//! #[derive(BorshSerialize, BorshDeserialize)]
//! pub struct MyToken;
//!
//! #[derive(BorshSerialize, BorshDeserialize)]
//! pub struct InitMsg { name: String, symbol: String, decimals: u8, initial_supply: u128 }
//!
//! impl Contract for MyToken {
//!     type Init = InitMsg;
//!     type Exec = Exec;
//!     type Query = Query;
//!     fn init(ctx: &Context, msg: InitMsg) -> Self {
//!         Ownable::init(&ctx.sender()).unwrap();
//!         Norn20::init(&msg.name, &msg.symbol, msg.decimals).unwrap();
//!         if msg.initial_supply > 0 {
//!             Norn20::mint(&ctx.sender(), msg.initial_supply).unwrap();
//!         }
//!         MyToken
//!     }
//!     fn execute(&mut self, ctx: &Context, msg: Exec) -> ContractResult {
//!         match msg {
//!             Exec::Transfer { to, amount } => Norn20::transfer(ctx, &to, amount),
//!         }
//!     }
//!     fn query(&self, _ctx: &Context, msg: Query) -> ContractResult {
//!         match msg {
//!             Query::Balance { addr } => ok(Norn20::balance_of(&addr)),
//!         }
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

// -- SDK v3 standard library --
pub mod stdlib;

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
