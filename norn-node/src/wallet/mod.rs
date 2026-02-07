pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod format;
pub mod keystore;
pub mod prompt;
pub mod rpc_client;

use cli::WalletCommand;
use error::WalletError;

/// Run a wallet subcommand.
pub async fn run(command: WalletCommand) -> Result<(), WalletError> {
    match command {
        WalletCommand::Create { name, passphrase } => {
            commands::create::run(&name, passphrase.as_deref())
        }
        WalletCommand::Import {
            mnemonic,
            private_key,
            name,
            passphrase,
        } => commands::import::run(
            mnemonic,
            private_key.as_deref(),
            &name,
            passphrase.as_deref(),
        ),
        WalletCommand::Export {
            name,
            show_mnemonic,
            show_private_key,
        } => commands::export::run(name.as_deref(), show_mnemonic, show_private_key),
        WalletCommand::List { json } => commands::list::run(json),
        WalletCommand::Use { name } => commands::use_wallet::run(&name),
        WalletCommand::Delete { name, force } => commands::delete::run(&name, force),
        WalletCommand::Address { name, json } => commands::address::run(name.as_deref(), json),
        WalletCommand::Balance {
            address,
            token,
            json,
            rpc_url,
        } => {
            commands::balance::run(
                address.as_deref(),
                token.as_deref(),
                json,
                rpc_url.as_deref(),
            )
            .await
        }
        WalletCommand::Transfer {
            to,
            amount,
            token,
            memo,
            yes,
            rpc_url,
        } => {
            commands::transfer::run(
                &to,
                &amount,
                token.as_deref(),
                memo.as_deref(),
                yes,
                rpc_url.as_deref(),
            )
            .await
        }
        WalletCommand::Register { name, rpc_url } => {
            commands::register::run(name.as_deref(), rpc_url.as_deref()).await
        }
        WalletCommand::Commit { name, rpc_url } => {
            commands::commit::run(name.as_deref(), rpc_url.as_deref()).await
        }
        WalletCommand::Status {
            name,
            json,
            rpc_url,
        } => commands::status::run(name.as_deref(), json, rpc_url.as_deref()).await,
        WalletCommand::History {
            limit,
            json,
            rpc_url,
        } => commands::history::run(limit, json, rpc_url.as_deref()).await,
        WalletCommand::Faucet { address, rpc_url } => {
            commands::faucet::run(address.as_deref(), rpc_url.as_deref()).await
        }
        WalletCommand::Block {
            height,
            json,
            rpc_url,
        } => commands::block::run(height.as_deref(), json, rpc_url.as_deref()).await,
        WalletCommand::WeaveState { json, rpc_url } => {
            commands::weave_state::run(json, rpc_url.as_deref()).await
        }
        WalletCommand::Config {
            rpc_url,
            network,
            json,
        } => commands::config_cmd::run(rpc_url.as_deref(), network.as_deref(), json),
        WalletCommand::RegisterName { name, yes, rpc_url } => {
            commands::register_name::run(&name, yes, rpc_url.as_deref()).await
        }
        WalletCommand::Resolve {
            name,
            json,
            rpc_url,
        } => commands::resolve::run(&name, json, rpc_url.as_deref()).await,
        WalletCommand::Names { json, rpc_url } => {
            commands::names::run(json, rpc_url.as_deref()).await
        }
    }
}
