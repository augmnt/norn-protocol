use norn_crypto::address::pubkey_to_address;
use norn_thread::state::compute_state_hash;
use norn_types::thread::ThreadState;
use norn_types::weave::Registration;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, print_error, print_success, style_bold};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::prompt_password;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(name: Option<&str>, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = match name {
        Some(n) => n,
        None => config.active_wallet_name()?,
    };

    let ks = Keystore::load(wallet_name)?;
    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;

    let address = pubkey_to_address(&keypair.public_key());
    let initial_state = ThreadState::new();
    let state_hash = compute_state_hash(&initial_state);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Build signing bytes (same as weave registration validation)
    let mut sig_data = Vec::new();
    sig_data.extend_from_slice(&address);
    sig_data.extend_from_slice(&keypair.public_key());
    sig_data.extend_from_slice(&state_hash);
    sig_data.extend_from_slice(&now.to_le_bytes());

    let signature = keypair.sign(&sig_data);

    let registration = Registration {
        thread_id: address,
        owner: keypair.public_key(),
        initial_state_hash: state_hash,
        timestamp: now,
        signature,
    };

    // Serialize and submit
    let bytes =
        borsh::to_vec(&registration).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let hex_data = hex::encode(&bytes);

    println!();
    println!(
        "  {} {}",
        style_bold().apply_to("Registering thread for"),
        format_address(&address)
    );

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;
    let result = rpc.submit_registration(&hex_data).await?;

    if result.success {
        print_success("Thread registered successfully!");
    } else {
        print_error(
            &format!(
                "Registration failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            Some("Your thread may already be registered. Check with `norn-node wallet status`."),
        );
    }
    println!();

    Ok(())
}
