use norn_types::loom::{LoomRegistration, LOOM_DEPLOY_FEE};
use norn_types::primitives::NATIVE_TOKEN_ID;

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
    // Validate name locally.
    norn_types::loom::validate_loom_name(name).map_err(|e| WalletError::Other(e.to_string()))?;

    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Check balance for deploy fee.
    let addr_hex = hex::encode(ks.address);
    let token_hex = hex::encode(NATIVE_TOKEN_ID);
    let balance_str = rpc.get_balance(&addr_hex, &token_hex).await?;
    let current_balance: u128 = balance_str.parse().unwrap_or(0);

    if current_balance < LOOM_DEPLOY_FEE {
        return Err(WalletError::InsufficientBalance {
            available: format_amount_with_symbol(current_balance, &NATIVE_TOKEN_ID),
            required: format_amount_with_symbol(LOOM_DEPLOY_FEE, &NATIVE_TOKEN_ID),
        });
    }

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Deploy Loom"));
        print_divider();
        println!("  Name:     {}", style_info().apply_to(name));
        println!(
            "  Operator: {} ({})",
            format_address(&ks.address),
            wallet_name
        );
        println!(
            "  Fee:      {}",
            style_bold().apply_to(format_amount_with_symbol(LOOM_DEPLOY_FEE, &NATIVE_TOKEN_ID))
        );
        println!(
            "  Balance:  {}",
            style_dim().apply_to(format_amount_with_symbol(current_balance, &NATIVE_TOKEN_ID))
        );
        println!();

        if !confirm("Deploy this loom?")? {
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

    // Build LoomConfig. The loom_id will be computed from the registration data.
    let loom_config = norn_types::loom::LoomConfig {
        loom_id: [0u8; 32], // placeholder — computed by consensus
        name: name.to_string(),
        max_participants: 1000,
        min_participants: 1,
        accepted_tokens: vec![NATIVE_TOKEN_ID],
        config_data: vec![],
    };

    let mut loom_reg = LoomRegistration {
        config: loom_config,
        operator: keypair.public_key(),
        timestamp: now,
        signature: [0u8; 64],
    };

    let sig_data = norn_types::loom::loom_deploy_signing_data(&loom_reg);
    loom_reg.signature = keypair.sign(&sig_data);

    let bytes =
        borsh::to_vec(&loom_reg).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let hex_data = hex::encode(&bytes);

    let result = rpc.deploy_loom(&hex_data).await?;

    if result.success {
        let loom_id = norn_types::loom::compute_loom_id(&loom_reg);
        print_success(&format!("Loom '{}' deployed!", name));
        println!(
            "  {}",
            style_dim().apply_to(format!("Loom ID: {}", hex::encode(loom_id)))
        );
        let remaining = current_balance - LOOM_DEPLOY_FEE;
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
                "Loom deployment failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
