use borsh::BorshSerialize;

use norn_crypto::address::pubkey_to_address;
use norn_thread::state::compute_state_hash;
use norn_types::thread::ThreadState;
use norn_types::weave::CommitmentUpdate;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, print_error, print_success, style_bold};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::prompt_password;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(name: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = match name {
        Some(n) => n,
        None => config.active_wallet_name()?,
    };

    let ks = Keystore::load(wallet_name)?;
    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;

    let address = pubkey_to_address(&keypair.public_key());
    let state = ThreadState::new();
    let state_hash = compute_state_hash(&state);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Build a commitment update.
    // In a full implementation, we'd query the node for current version and prev hash.
    let mut commitment = CommitmentUpdate {
        thread_id: address,
        owner: keypair.public_key(),
        version: 0,
        state_hash,
        prev_commitment_hash: [0u8; 32],
        knot_count: 0,
        timestamp: now,
        signature: [0u8; 64],
    };

    // Sign the commitment
    let mut sig_data = Vec::new();
    commitment
        .thread_id
        .serialize(&mut sig_data)
        .expect("serialize");
    commitment
        .owner
        .serialize(&mut sig_data)
        .expect("serialize");
    commitment
        .version
        .serialize(&mut sig_data)
        .expect("serialize");
    commitment
        .state_hash
        .serialize(&mut sig_data)
        .expect("serialize");
    commitment
        .prev_commitment_hash
        .serialize(&mut sig_data)
        .expect("serialize");
    commitment
        .knot_count
        .serialize(&mut sig_data)
        .expect("serialize");
    commitment
        .timestamp
        .serialize(&mut sig_data)
        .expect("serialize");
    commitment.signature = keypair.sign(&sig_data);

    let bytes =
        borsh::to_vec(&commitment).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let hex_data = hex::encode(&bytes);

    println!();
    println!(
        "  {} {}",
        style_bold().apply_to("Committing thread state for"),
        format_address(&address)
    );

    let rpc = RpcClient::new(&config.rpc_url)?;
    let result = rpc.submit_commitment(&hex_data).await?;

    if result.success {
        print_success("Commitment submitted successfully!");
    } else {
        print_error(
            &format!(
                "Commitment failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            Some("Ensure your thread is registered first."),
        );
    }
    println!();

    Ok(())
}
