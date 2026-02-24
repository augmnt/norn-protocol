use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, print_error, print_success, style_bold, style_dim};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::prompt_password;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(loom_id: &str, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    println!();
    println!("  {}", style_bold().apply_to("Leave Loom"));
    println!("  Loom ID:     {}", style_dim().apply_to(loom_id));
    println!(
        "  Participant: {} ({})",
        format_address(&ks.address),
        wallet_name
    );
    println!();

    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;

    let participant_hex = hex::encode(ks.address);
    let pubkey_hex = hex::encode(keypair.public_key());

    // Sign: blake3(b"norn_leave_loom" || loom_id || address)
    let loom_id_bytes = hex::decode(loom_id)
        .map_err(|e| WalletError::Other(format!("invalid loom_id hex: {}", e)))?;
    let signing_msg =
        norn_crypto::hash::blake3_hash_multi(&[b"norn_leave_loom", &loom_id_bytes, &ks.address]);
    let signature = keypair.sign(&signing_msg);
    let signature_hex = hex::encode(signature);

    let result = rpc
        .leave_loom(loom_id, &participant_hex, &signature_hex, &pubkey_hex)
        .await?;

    if result.success {
        print_success("Left loom.");
    } else {
        print_error(
            &format!(
                "Failed to leave loom: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
