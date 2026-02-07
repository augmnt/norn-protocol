use norn_types::name::NAME_REGISTRATION_FEE;
use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::state_manager::validate_name;
use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount_with_symbol, print_divider, print_error, print_success,
    style_bold, style_dim, style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(name: &str, yes: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    // Validate name format locally first.
    validate_name(name).map_err(|e| WalletError::Other(e.to_string()))?;

    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Check if name is already taken.
    if let Some(resolution) = rpc.resolve_name(name).await? {
        print_error(
            &format!(
                "name '{}' is already registered by {}",
                name, resolution.owner
            ),
            None,
        );
        return Ok(());
    }

    // Check balance.
    let addr_hex = hex::encode(ks.address);
    let token_hex = hex::encode(NATIVE_TOKEN_ID);
    let balance_str = rpc.get_balance(&addr_hex, &token_hex).await?;
    let current_balance: u128 = balance_str.parse().unwrap_or(0);
    let fee = NAME_REGISTRATION_FEE;

    if current_balance < fee {
        return Err(WalletError::InsufficientBalance {
            available: format_amount_with_symbol(current_balance, &NATIVE_TOKEN_ID),
            required: format_amount_with_symbol(fee, &NATIVE_TOKEN_ID),
        });
    }

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Register Name"));
        print_divider();
        println!("  Name:    {}", style_info().apply_to(name));
        println!(
            "  Owner:   {} ({})",
            format_address(&ks.address),
            wallet_name
        );
        println!(
            "  Fee:     {}",
            style_bold().apply_to(format_amount_with_symbol(fee, &NATIVE_TOKEN_ID))
        );
        println!(
            "  Balance: {}",
            style_dim().apply_to(format_amount_with_symbol(current_balance, &NATIVE_TOKEN_ID))
        );
        println!();

        if !confirm("Register this name?")? {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;
    let sender_addr = norn_crypto::address::pubkey_to_address(&keypair.public_key());

    // Build a consensus-level NameRegistration (signed by the wallet keypair).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut name_reg = norn_types::weave::NameRegistration {
        name: name.to_string(),
        owner: sender_addr,
        owner_pubkey: keypair.public_key(),
        timestamp: now,
        fee_paid: fee,
        signature: [0u8; 64],
    };

    // Sign the name registration data.
    let sig_data = norn_weave::name::name_registration_signing_data(&name_reg);
    name_reg.signature = keypair.sign(&sig_data);

    let nr_bytes =
        borsh::to_vec(&name_reg).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let nr_hex = hex::encode(&nr_bytes);

    let result = rpc.register_name(name, &addr_hex, &nr_hex).await?;

    if result.success {
        print_success(&format!(
            "Name '{}' submitted for registration (will be included in next block)",
            name
        ));
        let remaining = current_balance - fee;
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
                "Name registration failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
