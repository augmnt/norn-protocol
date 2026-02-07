use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, style_bold, style_dim, style_success, style_warn};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(name: Option<&str>, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = match name {
        Some(n) => n,
        None => config.active_wallet_name()?,
    };

    let ks = Keystore::load(wallet_name)?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let thread_id = hex::encode(ks.address);
    let thread_info = rpc.get_thread(&thread_id).await?;

    if json {
        let info = serde_json::json!({
            "wallet": wallet_name,
            "address": format_address(&ks.address),
            "thread": thread_info,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {} {}", style_bold().apply_to("Wallet:"), wallet_name);
    println!("  Address: {}", format_address(&ks.address));

    match thread_info {
        Some(info) => {
            println!("  Status:  {}", style_success().apply_to("Registered"));
            println!("  Version: {}", info.version);
            println!(
                "  State:   {}",
                if info.state_hash.len() > 16 {
                    format!(
                        "{}...{}",
                        &info.state_hash[..8],
                        &info.state_hash[info.state_hash.len() - 8..]
                    )
                } else {
                    info.state_hash
                }
            );
        }
        None => {
            println!("  Status:  {}", style_warn().apply_to("Not registered"));
            println!(
                "  {}",
                style_dim().apply_to("Run `norn-node wallet register` to register your thread.")
            );
        }
    }
    println!();

    Ok(())
}
