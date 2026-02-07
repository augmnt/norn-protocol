use clap::{Parser, Subcommand};

use crate::error::NodeError;
use crate::wallet::cli::WalletCommand;

#[derive(Parser)]
#[command(name = "norn-node", about = "Norn Chain Node")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the node
    Run {
        /// Path to config file
        #[arg(short, long, default_value = "norn.toml")]
        config: String,
        /// Start in dev mode (solo validator, testnet faucet, auto-config)
        #[arg(long)]
        dev: bool,
    },
    /// Initialize a new node configuration
    Init {
        /// Output directory
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
    /// Generate genesis block
    Genesis {
        /// Path to genesis config file
        #[arg(short, long)]
        config: String,
        /// Output path for genesis block
        #[arg(short, long, default_value = "genesis.json")]
        output: String,
    },
    /// Generate a new keypair
    Keygen {
        /// Optional mnemonic passphrase
        #[arg(short, long)]
        passphrase: Option<String>,
    },
    /// Wallet management and operations
    Wallet {
        #[command(subcommand)]
        command: WalletCommand,
    },
}

pub async fn run(cli: Cli) -> Result<(), NodeError> {
    match cli.command {
        Command::Run { config, dev } => {
            let config = if dev {
                tracing::info!("Starting in dev mode (solo validator, memory storage)");
                let mut cfg = crate::config::NodeConfig::default();
                cfg.validator.enabled = true;
                cfg.validator.solo_mode = true;
                cfg.rpc.enabled = true;
                cfg.rpc.listen_addr = "127.0.0.1:9741".to_string();
                cfg.storage.db_type = "memory".to_string();
                // No keypair_seed — will auto-generate.
                cfg
            } else {
                crate::config::NodeConfig::load(&config)?
            };
            let mut node = crate::node::Node::new(config).await?;
            node.run().await
        }
        Command::Init { dir } => {
            crate::config::NodeConfig::init(&dir)?;
            tracing::info!("Node configuration initialized in {}", dir);
            Ok(())
        }
        Command::Genesis { config, output } => {
            crate::genesis::generate_genesis(&config, &output)?;
            tracing::info!("Genesis block written to {}", output);
            Ok(())
        }
        Command::Keygen { passphrase } => {
            let mnemonic = norn_crypto::seed::generate_mnemonic();
            println!("Mnemonic: {}", mnemonic);
            let seed =
                norn_crypto::seed::mnemonic_to_seed(&mnemonic, passphrase.as_deref().unwrap_or(""));
            let keypair = norn_crypto::hd::derive_default_keypair(&seed).map_err(|e| {
                NodeError::ConfigError {
                    reason: e.to_string(),
                }
            })?;
            let address = norn_crypto::address::pubkey_to_address(&keypair.public_key());
            println!("Public key: {}", hex::encode(keypair.public_key()));
            println!("Address: {}", hex::encode(address));
            Ok(())
        }
        Command::Wallet { command } => {
            crate::wallet::run(command)
                .await
                .map_err(|e| NodeError::ConfigError {
                    reason: e.to_string(),
                })
        }
    }
}
