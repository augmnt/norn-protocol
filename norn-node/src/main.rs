use clap::Parser;
use tracing_subscriber::EnvFilter;

mod banner;
mod cli;
mod config;
mod error;
mod genesis;
mod metrics;
mod node;
mod rpc;
mod state_manager;
mod state_store;
mod wallet;

/// Build a `norn_types::loom::Loom` from a `LoomRegistration` for registering
/// with the `LoomManager` at block-application time.
fn loom_from_registration(
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

fn main() {
    // Initialize tracing with configurable level via RUST_LOG env var.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = cli::Cli::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        if let Err(e) = cli::run(cli).await {
            tracing::error!("Fatal error: {}", e);
            std::process::exit(1);
        }
    });
}
