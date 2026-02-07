use clap::{Parser, Subcommand};

use crate::error::NodeError;
use crate::wallet::cli::WalletCommand;

#[derive(Parser)]
#[command(
    name = "norn",
    about = "Norn Protocol Node — Thread-based L1 with off-chain execution",
    version
)]
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
        /// Override RPC listen address (e.g., "0.0.0.0:9741" for LAN access)
        #[arg(long)]
        rpc_addr: Option<String>,
        /// Storage backend for dev mode: "memory" (default), "sqlite", "rocksdb"
        #[arg(long)]
        storage: Option<String>,
        /// Network: "dev" (default for --dev), "testnet", "mainnet"
        #[arg(long)]
        network: Option<String>,
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
        Command::Run {
            config,
            dev,
            rpc_addr,
            storage,
            network,
        } => {
            crate::banner::print_banner();

            let mut config = if dev {
                let mut cfg = crate::config::NodeConfig::default();
                cfg.validator.enabled = true;
                cfg.validator.solo_mode = true;
                cfg.rpc.enabled = true;
                cfg.rpc.listen_addr = "127.0.0.1:9741".to_string();
                cfg.storage.db_type = "memory".to_string();
                cfg.network.listen_addr = "0.0.0.0:9740".to_string();
                cfg.network_id = "dev".to_string();
                let (devnet_config, _, _) = crate::genesis::devnet_genesis();
                cfg.genesis_config = Some(devnet_config);
                cfg
            } else {
                crate::config::NodeConfig::load(&config)?
            };

            // Apply CLI overrides.
            if let Some(addr) = rpc_addr {
                config.rpc.listen_addr = addr;
            }
            if let Some(db) = storage {
                config.storage.db_type = db;
            }
            if let Some(ref net) = network {
                config.network_id = net.clone();
            }

            // Parse and validate network ID.
            let network_id =
                norn_types::network::NetworkId::parse(&config.network_id).ok_or_else(|| {
                    NodeError::ConfigError {
                        reason: format!(
                            "unknown network '{}', expected 'dev', 'testnet', or 'mainnet'",
                            config.network_id
                        ),
                    }
                })?;

            // Print compact startup summary.
            {
                let dim = console::Style::new().dim();
                let cyan = console::Style::new().cyan();
                let mode = if dev {
                    format!("dev · solo validator · {} storage", config.storage.db_type)
                } else {
                    "config".to_string()
                };
                println!(
                    "  {} {} · {}",
                    dim.apply_to("Network "),
                    cyan.apply_to(network_id.as_str()),
                    cyan.apply_to(network_id.chain_id()),
                );
                println!(
                    "  {}  {} (P2P) | {} (RPC)",
                    dim.apply_to("Listen  "),
                    cyan.apply_to(&config.network.listen_addr),
                    cyan.apply_to(&config.rpc.listen_addr),
                );
                println!("  {}  {}", dim.apply_to("Mode    "), cyan.apply_to(mode),);
                if dev {
                    let (_, founder_addr, _) = crate::genesis::devnet_genesis();
                    let green = console::Style::new().green();
                    println!(
                        "  {} 0x{} (10,000,000 NORN)",
                        dim.apply_to("Founder "),
                        green.apply_to(hex::encode(founder_addr)),
                    );
                    println!(
                        "  {}  blake3(b\"augmnt-devnet-founder\")",
                        dim.apply_to("Seed    "),
                    );
                }
                println!();
            }

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
