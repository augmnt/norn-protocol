use norn_types::name::{ALLOWED_RECORD_KEYS, MAX_RECORD_VALUE_LEN};

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, print_divider, print_error, print_success, style_bold, style_dim, style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(
    name: &str,
    key: &str,
    value: &str,
    yes: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
    // Validate key locally.
    if !ALLOWED_RECORD_KEYS.contains(&key) {
        print_error(
            &format!("invalid record key '{}'", key),
            Some(&format!("Allowed keys: {}", ALLOWED_RECORD_KEYS.join(", "))),
        );
        return Ok(());
    }

    // Validate value length locally.
    if value.len() > MAX_RECORD_VALUE_LEN {
        print_error(
            &format!(
                "value too long ({} bytes, max {})",
                value.len(),
                MAX_RECORD_VALUE_LEN
            ),
            None,
        );
        return Ok(());
    }

    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Verify name exists and is owned by this wallet.
    let resolution = rpc.resolve_name(name).await?;
    let resolution = match resolution {
        Some(r) => r,
        None => {
            print_error(&format!("name '{}' is not registered", name), None);
            return Ok(());
        }
    };

    let owner_hex = hex::encode(ks.address);
    if resolution.owner != owner_hex {
        print_error(
            &format!(
                "name '{}' is owned by {}, not this wallet",
                name, resolution.owner
            ),
            None,
        );
        return Ok(());
    }

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Set NNS Record"));
        print_divider();
        println!("  Name:   {}", style_info().apply_to(name));
        println!("  Key:    {}", style_info().apply_to(key));
        println!("  Value:  {}", value);
        println!(
            "  Owner:  {} ({})",
            format_address(&ks.address),
            wallet_name
        );
        println!();

        if !confirm("Set this record?")? {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;
    let sender_addr = norn_crypto::address::pubkey_to_address(&keypair.public_key());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut update = norn_types::weave::NameRecordUpdate {
        name: name.to_string(),
        key: key.to_string(),
        value: value.to_string(),
        owner: sender_addr,
        owner_pubkey: keypair.public_key(),
        timestamp: now,
        signature: [0u8; 64],
    };

    let sig_data = norn_weave::name::name_record_update_signing_data(&update);
    update.signature = keypair.sign(&sig_data);

    let update_bytes =
        borsh::to_vec(&update).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let update_hex = hex::encode(&update_bytes);

    let result = rpc
        .set_name_record(name, key, value, &owner_hex, &update_hex)
        .await?;

    if result.success {
        print_success(&format!("Record '{}' set on name '{}'", key, name));
        println!(
            "  {}",
            style_dim().apply_to("Will be included in next block")
        );
    } else {
        print_error(
            &format!(
                "Set record failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
