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
    to_hex: &str,
    yes: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
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

    // Validate recipient address.
    let to_bytes: [u8; 20] = hex::decode(to_hex)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| {
            WalletError::Other("invalid recipient address (expected 40 hex chars)".to_string())
        })?;

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Transfer Name (NNS)"));
        print_divider();
        println!("  Name:  {}", style_info().apply_to(name));
        println!("  From:  {} ({})", format_address(&ks.address), wallet_name);
        println!(
            "  To:    {}",
            style_info().apply_to(format_address(&to_bytes))
        );
        println!("  Fee:   {}", style_dim().apply_to("free"));
        println!();

        if !confirm("Transfer this name?")? {
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

    let mut transfer = norn_types::weave::NameTransfer {
        name: name.to_string(),
        from: sender_addr,
        from_pubkey: keypair.public_key(),
        to: to_bytes,
        timestamp: now,
        signature: [0u8; 64],
    };

    let sig_data = norn_weave::name::name_transfer_signing_data(&transfer);
    transfer.signature = keypair.sign(&sig_data);

    let transfer_bytes =
        borsh::to_vec(&transfer).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let transfer_hex = hex::encode(&transfer_bytes);

    let result = rpc.transfer_name(name, &owner_hex, &transfer_hex).await?;

    if result.success {
        print_success(&format!(
            "Name '{}' transfer submitted (will be included in next block)",
            name
        ));
        println!(
            "  {}",
            style_dim().apply_to(format!("New owner: {}", format_address(&to_bytes)))
        );
    } else {
        print_error(
            &format!(
                "Name transfer failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}
