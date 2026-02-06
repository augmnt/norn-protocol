use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_success, style_bold};

pub fn run(rpc_url: Option<&str>, json: bool) -> Result<(), WalletError> {
    let mut config = WalletConfig::load()?;

    if let Some(url) = rpc_url {
        config.rpc_url = url.to_string();
        config.save()?;
        print_success(&format!("RPC URL set to {}", url));
        return Ok(());
    }

    // Show current config
    if json {
        let info = serde_json::json!({
            "active_wallet": config.active_wallet,
            "rpc_url": config.rpc_url,
            "wallets": config.wallets,
            "data_dir": WalletConfig::data_dir()?.to_string_lossy(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {}", style_bold().apply_to("Wallet Configuration"));
    println!(
        "  Active wallet: {}",
        config.active_wallet.as_deref().unwrap_or("(none)")
    );
    println!("  RPC URL:       {}", config.rpc_url);
    println!("  Data dir:      {}", WalletConfig::data_dir()?.display());
    println!("  Wallets:       {}", config.wallets.len());
    println!();

    Ok(())
}
