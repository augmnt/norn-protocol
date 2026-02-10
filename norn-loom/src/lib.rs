//! Off-chain WebAssembly smart contract runtime for the Norn Protocol.
//!
//! Provides Loom lifecycle management, a Wasmtime-based execution engine with
//! host functions, gas metering, and on-chain dispute resolution.

pub mod call_stack;
pub mod dispute;
pub mod error;
pub mod gas;
pub mod host;
pub mod lifecycle;
pub mod runtime;
pub mod sdk;
pub mod state;
