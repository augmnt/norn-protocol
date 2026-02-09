//! SDK for writing Norn Protocol loom smart contracts.
//!
//! Contracts are compiled to `wasm32-unknown-unknown` and executed by the
//! norn-loom runtime. This crate provides safe Rust wrappers around the host
//! functions, an output buffer for returning data, and encoding helpers.
//!
//! # Example
//!
//! ```ignore
//! use norn_sdk::{host, output, encoding};
//!
//! #[no_mangle]
//! pub extern "C" fn init() {
//!     host::state_set(b"counter", &encoding::encode_u64(0));
//!     host::log("initialized");
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn execute(ptr: i32, len: i32) -> i32 {
//!     let input = output::read_input(ptr, len);
//!     // ... process input ...
//!     output::set_output(&encoding::encode_u64(42));
//!     0
//! }
//! ```

#![no_std]

extern crate alloc;

pub mod encoding;
pub mod host;
pub mod output;
