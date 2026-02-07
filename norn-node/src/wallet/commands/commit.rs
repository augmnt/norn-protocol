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

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Query the node for current thread version and state.
    let thread_id_hex = hex::encode(address);
    let (current_version, prev_hash, state_hash) = match rpc.get_thread(&thread_id_hex).await? {
        Some(info) => {
            let mut prev = [0u8; 32];
            if let Ok(bytes) = hex::decode(&info.state_hash) {
                if bytes.len() == 32 {
                    prev.copy_from_slice(&bytes);
                }
            }
            // Query thread state for the actual state hash.
            let sh = match rpc.get_thread_state(&thread_id_hex).await? {
                Some(ts_info) => {
                    let mut h = [0u8; 32];
                    if let Ok(bytes) = hex::decode(&ts_info.state_hash) {
                        if bytes.len() == 32 {
                            h.copy_from_slice(&bytes);
                        }
                    }
                    h
                }
                None => compute_state_hash(&ThreadState::new()),
            };
            (info.version, prev, sh)
        }
        None => {
            // Thread not registered yet — use genesis defaults.
            let state = ThreadState::new();
            let state_hash = compute_state_hash(&state);
            (0, [0u8; 32], state_hash)
        }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Build a commitment update with the real version from the node.
    let new_version = current_version + 1;
    let mut commitment = CommitmentUpdate {
        thread_id: address,
        owner: keypair.public_key(),
        version: new_version,
        state_hash,
        prev_commitment_hash: prev_hash,
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
        "  {} {} (version {})",
        style_bold().apply_to("Committing thread state for"),
        format_address(&address),
        new_version
    );

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
