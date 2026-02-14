//! Full node binary for the Norn Protocol.
//!
//! Provides the CLI interface, JSON-RPC server (jsonrpsee), wallet management
//! (24 subcommands with encrypted keystore), Prometheus metrics, node
//! configuration, and genesis state handling.

pub mod banner;
pub mod cli;
pub mod config;
pub mod error;
pub mod genesis;
pub mod metrics;
pub mod node;
pub mod rpc;
pub mod state_manager;
pub mod state_store;
pub mod wallet;

/// Build a `norn_types::loom::Loom` from a `LoomRegistration` for registering
/// with the `LoomManager` at block-application time.
pub fn loom_from_registration(
    ld: &norn_types::loom::LoomRegistration,
    loom_id: norn_types::primitives::LoomId,
) -> norn_types::loom::Loom {
    norn_types::loom::Loom {
        config: norn_types::loom::LoomConfig {
            loom_id,
            name: ld.config.name.clone(),
            max_participants: 1000,
            min_participants: 1,
            accepted_tokens: vec![norn_types::primitives::NATIVE_TOKEN_ID],
            config_data: vec![],
        },
        operator: ld.operator,
        participants: Vec::new(),
        state_hash: [0u8; 32],
        version: 0,
        active: true,
        last_updated: ld.timestamp,
    }
}
