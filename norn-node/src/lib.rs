//! Full node binary for the Norn Protocol.
//!
//! Provides the CLI interface, JSON-RPC server (jsonrpsee), wallet management
//! (17 subcommands with encrypted keystore), Prometheus metrics, node
//! configuration, and genesis state handling.

pub mod cli;
pub mod config;
pub mod error;
pub mod genesis;
pub mod metrics;
pub mod node;
pub mod rpc;
pub mod state_manager;
pub mod tools;
pub mod wallet;
