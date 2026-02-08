use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, print_divider, print_error, print_success, style_bold, style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(
    token: &str,
    amount: &str,
    yes: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
    let amount_val: u128 = amount
        .replace('_', "")
        .parse()
        .map_err(|_| WalletError::Other("invalid amount".to_string()))?;

    if amount_val == 0 {
        return Err(WalletError::Other("amount must be > 0".to_string()));
    }

    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Resolve token (by symbol or hex ID).
    let token_info = super::mint_token::resolve_token(&rpc, token).await?;
    let token_id = super::mint_token::hex_to_token_id(&token_info.token_id)?;

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Burn Tokens"));
        print_divider();
        println!(
            "  Token:   {} ({})",
            style_info().apply_to(&token_info.symbol),
            &token_info.token_id[..16]
        );
        println!(
            "  Burner:  {} ({})",
            format_address(&ks.address),
            wallet_name
        );
        println!("  Amount:  {}", style_bold().apply_to(amount));
        println!();

        if !confirm("Burn these tokens?")? {
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

    let burner = norn_crypto::address::pubkey_to_address(&keypair.public_key());

    let mut token_burn = norn_types::weave::TokenBurn {
        token_id,
        burner,
        burner_pubkey: keypair.public_key(),
        amount: amount_val,
        timestamp: now,
        signature: [0u8; 64],
    };

    let sig_data = norn_weave::token::token_burn_signing_data(&token_burn);
    token_burn.signature = keypair.sign(&sig_data);

    let bytes =
        borsh::to_vec(&token_burn).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let hex_data = hex::encode(&bytes);

    let result = rpc.burn_token(&hex_data).await?;

    if result.success {
        print_success(&format!("Burned {} {}", amount, token_info.symbol));
    } else {
        print_error(
            &format!(
                "Burn failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
