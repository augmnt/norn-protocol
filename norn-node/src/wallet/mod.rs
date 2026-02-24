pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod format;
pub mod keystore;
pub mod prompt;
pub mod rpc_client;
pub mod ui;

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
        WalletCommand::TransferName {
            name,
            to,
            yes,
            rpc_url,
        } => commands::transfer_name::run(&name, &to, yes, rpc_url.as_deref()).await,
        WalletCommand::ReverseName {
            address,
            json,
            rpc_url,
        } => commands::reverse_resolve::run(&address, json, rpc_url.as_deref()).await,
        WalletCommand::SetNameRecord {
            name,
            key,
            value,
            yes,
            rpc_url,
        } => commands::set_name_record::run(&name, &key, &value, yes, rpc_url.as_deref()).await,
        WalletCommand::NameRecords {
            name,
            json,
            rpc_url,
        } => commands::name_records::run(&name, json, rpc_url.as_deref()).await,
        WalletCommand::Names { json, rpc_url } => {
            commands::names::run(json, rpc_url.as_deref()).await
        }
        WalletCommand::NodeInfo { json, rpc_url } => {
            commands::node_info::run(json, rpc_url.as_deref()).await
        }
        WalletCommand::Fees { json, rpc_url } => {
            commands::fees::run(json, rpc_url.as_deref()).await
        }
        WalletCommand::Validators { json, rpc_url } => {
            commands::validators::run(json, rpc_url.as_deref()).await
        }
        WalletCommand::Whoami { json, rpc_url } => {
            commands::whoami::run(json, rpc_url.as_deref()).await
        }
        WalletCommand::SignMessage { message, name } => {
            commands::sign_message::run(&message, name.as_deref())
        }
        WalletCommand::VerifyMessage {
            message,
            signature,
            pubkey,
        } => commands::verify_message::run(&message, &signature, &pubkey),
        WalletCommand::Rename { from, to } => commands::rename::run(&from, &to),
        WalletCommand::ChangePassword { name } => commands::change_password::run(name.as_deref()),
        WalletCommand::CreateToken {
            name,
            symbol,
            decimals,
            max_supply,
            initial_supply,
            yes,
            rpc_url,
        } => {
            commands::create_token::run(
                &name,
                &symbol,
                decimals,
                &max_supply,
                &initial_supply,
                yes,
                rpc_url.as_deref(),
            )
            .await
        }
        WalletCommand::MintToken {
            token,
            to,
            amount,
            yes,
            rpc_url,
        } => commands::mint_token::run(&token, &to, &amount, yes, rpc_url.as_deref()).await,
        WalletCommand::BurnToken {
            token,
            amount,
            yes,
            rpc_url,
        } => commands::burn_token::run(&token, &amount, yes, rpc_url.as_deref()).await,
        WalletCommand::TokenInfo {
            token,
            json,
            rpc_url,
        } => commands::token_info::run(&token, json, rpc_url.as_deref()).await,
        WalletCommand::ListTokens {
            limit,
            json,
            rpc_url,
        } => commands::list_tokens::run(limit, json, rpc_url.as_deref()).await,
        WalletCommand::TokenBalances { json, rpc_url } => {
            commands::token_balances::run(json, rpc_url.as_deref()).await
        }
        WalletCommand::DeployLoom { name, yes, rpc_url } => {
            commands::deploy_loom::run(&name, yes, rpc_url.as_deref()).await
        }
        WalletCommand::LoomInfo {
            loom_id,
            json,
            rpc_url,
        } => commands::loom_info::run(&loom_id, json, rpc_url.as_deref()).await,
        WalletCommand::ListLooms {
            limit,
            json,
            rpc_url,
        } => commands::list_looms::run(limit, json, rpc_url.as_deref()).await,
        WalletCommand::UploadBytecode {
            loom_id,
            bytecode,
            rpc_url,
        } => commands::upload_bytecode::run(&loom_id, &bytecode, rpc_url.as_deref()).await,
        WalletCommand::ExecuteLoom {
            loom_id,
            input,
            rpc_url,
        } => commands::execute_loom::run(&loom_id, &input, rpc_url.as_deref()).await,
        WalletCommand::QueryLoom {
            loom_id,
            input,
            json,
            rpc_url,
        } => commands::query_loom::run(&loom_id, input.as_deref(), json, rpc_url.as_deref()).await,
        WalletCommand::JoinLoom { loom_id, rpc_url } => {
            commands::join_loom::run(&loom_id, rpc_url.as_deref()).await
        }
        WalletCommand::LeaveLoom { loom_id, rpc_url } => {
            commands::leave_loom::run(&loom_id, rpc_url.as_deref()).await
        }
        WalletCommand::NewLoom { name } => commands::new_loom::run(&name),
        WalletCommand::Stake {
            amount,
            yes,
            rpc_url,
        } => {
            let amount: u128 = amount.parse().map_err(|_| {
                crate::wallet::error::WalletError::Other("invalid amount".to_string())
            })?;
            commands::stake::run(amount, yes, rpc_url.as_deref()).await
        }
        WalletCommand::Unstake {
            amount,
            yes,
            rpc_url,
        } => {
            let amount: u128 = amount.parse().map_err(|_| {
                crate::wallet::error::WalletError::Other("invalid amount".to_string())
            })?;
            commands::unstake::run(amount, yes, rpc_url.as_deref()).await
        }
        WalletCommand::StakingInfo { validator, rpc_url } => {
            commands::staking_info::run(validator.as_deref(), rpc_url.as_deref()).await
        }
        WalletCommand::Rewards { json, rpc_url } => {
            commands::rewards::run(json, rpc_url.as_deref()).await
        }
    }
}
