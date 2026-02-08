use norn_types::primitives::NATIVE_TOKEN_ID;
use norn_types::token::TOKEN_CREATION_FEE;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount_with_symbol, print_divider, print_error, print_success,
    style_bold, style_dim, style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};
use crate::wallet::rpc_client::RpcClient;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    name: &str,
    symbol: &str,
    decimals: u8,
    max_supply: &str,
    initial_supply: &str,
    yes: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
    // Validate inputs locally.
    norn_types::token::validate_token_name(name).map_err(|e| WalletError::Other(e.to_string()))?;
    norn_types::token::validate_token_symbol(symbol)
        .map_err(|e| WalletError::Other(e.to_string()))?;
    if decimals > norn_types::token::MAX_TOKEN_DECIMALS {
        return Err(WalletError::Other(format!(
            "decimals must be <= {}",
            norn_types::token::MAX_TOKEN_DECIMALS
        )));
    }

    let max_supply_val: u128 = max_supply
        .replace('_', "")
        .parse()
        .map_err(|_| WalletError::Other("invalid max-supply value".to_string()))?;
    let initial_supply_val: u128 = initial_supply
        .replace('_', "")
        .parse()
        .map_err(|_| WalletError::Other("invalid initial-supply value".to_string()))?;

    if max_supply_val > 0 && initial_supply_val > max_supply_val {
        return Err(WalletError::Other(
            "initial supply cannot exceed max supply".to_string(),
        ));
    }

    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Check if symbol is already taken.
    if let Some(existing) = rpc.get_token_by_symbol(symbol).await? {
        print_error(
            &format!(
                "symbol '{}' is already registered (token: {})",
                symbol, existing.token_id
            ),
            None,
        );
        return Ok(());
    }

    // Check balance for creation fee.
    let addr_hex = hex::encode(ks.address);
    let token_hex = hex::encode(NATIVE_TOKEN_ID);
    let balance_str = rpc.get_balance(&addr_hex, &token_hex).await?;
    let current_balance: u128 = balance_str.parse().unwrap_or(0);

    if current_balance < TOKEN_CREATION_FEE {
        return Err(WalletError::InsufficientBalance {
            available: format_amount_with_symbol(current_balance, &NATIVE_TOKEN_ID),
            required: format_amount_with_symbol(TOKEN_CREATION_FEE, &NATIVE_TOKEN_ID),
        });
    }

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Create Token"));
        print_divider();
        println!("  Name:           {}", style_info().apply_to(name));
        println!("  Symbol:         {}", style_info().apply_to(symbol));
        println!("  Decimals:       {}", decimals);
        println!(
            "  Max Supply:     {}",
            if max_supply_val == 0 {
                "unlimited".to_string()
            } else {
                max_supply_val.to_string()
            }
        );
        println!("  Initial Supply: {}", initial_supply_val);
        println!(
            "  Creator:        {} ({})",
            format_address(&ks.address),
            wallet_name
        );
        println!(
            "  Fee:            {}",
            style_bold().apply_to(format_amount_with_symbol(
                TOKEN_CREATION_FEE,
                &NATIVE_TOKEN_ID
            ))
        );
        println!(
            "  Balance:        {}",
            style_dim().apply_to(format_amount_with_symbol(current_balance, &NATIVE_TOKEN_ID))
        );
        println!();

        if !confirm("Create this token?")? {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let creator = norn_crypto::address::pubkey_to_address(&keypair.public_key());

    let mut token_def = norn_types::weave::TokenDefinition {
        name: name.to_string(),
        symbol: symbol.to_uppercase(),
        decimals,
        max_supply: max_supply_val,
        initial_supply: initial_supply_val,
        creator,
        creator_pubkey: keypair.public_key(),
        timestamp: now,
        signature: [0u8; 64],
    };

    let sig_data = norn_weave::token::token_definition_signing_data(&token_def);
    token_def.signature = keypair.sign(&sig_data);

    let bytes =
        borsh::to_vec(&token_def).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let hex_data = hex::encode(&bytes);

    let result = rpc.create_token(&hex_data).await?;

    if result.success {
        let token_id = norn_types::token::compute_token_id(
            &token_def.creator,
            &token_def.name,
            &token_def.symbol,
            token_def.decimals,
            token_def.max_supply,
            token_def.timestamp,
        );
        print_success(&format!("Token '{}' ({}) created!", name, symbol));
        println!(
            "  {}",
            style_dim().apply_to(format!("Token ID: {}", hex::encode(token_id)))
        );
        let remaining = current_balance - TOKEN_CREATION_FEE;
        println!(
            "  {}",
            style_dim().apply_to(format!(
                "Remaining balance: {}",
                format_amount_with_symbol(remaining, &NATIVE_TOKEN_ID)
            ))
        );
    } else {
        print_error(
            &format!(
                "Token creation failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
