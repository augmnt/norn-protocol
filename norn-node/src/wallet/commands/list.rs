use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, style_bold, style_dim, style_success};
use crate::wallet::keystore::Keystore;

pub fn run(json: bool) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let names = Keystore::list_names()?;

    if names.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("  No wallets found.");
            println!(
                "  {}",
                style_dim().apply_to("Create one with: norn-node wallet create --name <NAME>")
            );
        }
        return Ok(());
    }

    if json {
        let wallets: Vec<serde_json::Value> = names
            .iter()
            .filter_map(|name| {
                let ks = Keystore::load(name).ok()?;
                Some(serde_json::json!({
                    "name": name,
                    "address": format_address(&ks.address),
                    "active": config.active_wallet.as_deref() == Some(name.as_str()),
                }))
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&wallets).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {}", style_bold().apply_to("Wallets"));
    println!(
        "  {}",
        style_dim().apply_to("────────────────────────────────────────────────────")
    );

    for name in &names {
        let active = config.active_wallet.as_deref() == Some(name.as_str());
        let marker = if active {
            style_success().apply_to("▸ ").to_string()
        } else {
            "  ".to_string()
        };

        match Keystore::load(name) {
            Ok(ks) => {
                let active_label = if active { " (active)" } else { "" };
                println!(
                    "  {}{:<16} {}{}",
                    marker,
                    style_bold().apply_to(name),
                    format_address(&ks.address),
                    style_dim().apply_to(active_label)
                );
            }
            Err(_) => {
                println!("  {}{:<16} (error loading)", marker, name);
            }
        }
    }
    println!();

    Ok(())
}
