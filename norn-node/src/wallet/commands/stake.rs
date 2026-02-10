use norn_types::weave::StakeOperation;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount_with_symbol, print_divider, print_success, style_bold, style_dim,
    style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};
use crate::wallet::rpc_client::RpcClient;
use norn_types::primitives::NATIVE_TOKEN_ID;

pub async fn run(amount: u128, yes: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    if amount == 0 {
        return Err(WalletError::Other(
            "stake amount must be positive".to_string(),
        ));
    }

    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Stake Tokens"));
        print_divider();
        println!(
            "  Amount:    {}",
            style_info().apply_to(format_amount_with_symbol(amount, &NATIVE_TOKEN_ID))
        );
        println!(
            "  Validator: {} ({})",
            format_address(&ks.address),
            wallet_name
        );
        println!(
            "  {}",
            style_dim().apply_to("Tokens will be locked as validator stake")
        );
        println!();

        if !confirm("Stake these tokens?")? {
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

    let mut op = StakeOperation::Stake {
        pubkey: keypair.public_key(),
        amount,
        timestamp: now,
        signature: [0u8; 64],
    };

    // Sign.
    let sig_data = norn_weave::staking::stake_operation_signing_data(&op);
    let signature = keypair.sign(&sig_data);
    match &mut op {
        StakeOperation::Stake { signature: s, .. } => *s = signature,
        _ => unreachable!(),
    }

    // Submit via RPC.
    let hex_data = hex::encode(borsh::to_vec(&op).map_err(|e| WalletError::Other(e.to_string()))?);
    let result = rpc.submit_stake(&hex_data).await?;

    if result.success {
        print_success("Stake operation submitted successfully");
    } else {
        return Err(WalletError::Other(
            result.reason.unwrap_or_else(|| "unknown error".to_string()),
        ));
    }

    Ok(())
}
