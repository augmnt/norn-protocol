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
