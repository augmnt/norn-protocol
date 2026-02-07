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
        } => commands::balance::run(address.as_deref(), token.as_deref(), json).await,
        WalletCommand::Transfer {
            to,
            amount,
            token,
            memo,
            yes,
        } => commands::transfer::run(&to, &amount, token.as_deref(), memo.as_deref(), yes).await,
        WalletCommand::Register { name } => commands::register::run(name.as_deref()).await,
        WalletCommand::Commit { name } => commands::commit::run(name.as_deref()).await,
        WalletCommand::Status { name, json } => commands::status::run(name.as_deref(), json).await,
        WalletCommand::History { limit, json } => commands::history::run(limit, json).await,
        WalletCommand::Faucet { address } => commands::faucet::run(address.as_deref()).await,
        WalletCommand::Block { height, json } => {
            commands::block::run(height.as_deref(), json).await
        }
        WalletCommand::WeaveState { json } => commands::weave_state::run(json).await,
        WalletCommand::Config { rpc_url, json } => {
            commands::config_cmd::run(rpc_url.as_deref(), json)
        }
        WalletCommand::RegisterName { name, yes } => commands::register_name::run(&name, yes).await,
        WalletCommand::Resolve { name, json } => commands::resolve::run(&name, json).await,
        WalletCommand::Names { json } => commands::names::run(json).await,
    }
}
