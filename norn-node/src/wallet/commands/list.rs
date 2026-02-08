use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, style_bold, style_dim, style_success};
use crate::wallet::keystore::Keystore;
use crate::wallet::ui::{data_table, print_table};

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
                style_dim().apply_to("Create one with: norn wallet create --name <NAME>")
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

    let mut table = data_table(&["Name", "Address", "Status"]);

    for name in &names {
        let active = config.active_wallet.as_deref() == Some(name.as_str());

        match Keystore::load(name) {
            Ok(ks) => {
                let status = if active {
                    format!("{} active", style_success().apply_to("\u{25cf}"))
                } else {
                    String::new()
                };
                table.add_row(vec![
                    comfy_table::Cell::new(name),
                    comfy_table::Cell::new(format_address(&ks.address)),
                    comfy_table::Cell::new(status),
                ]);
            }
            Err(_) => {
                table.add_row(vec![
                    comfy_table::Cell::new(name),
                    comfy_table::Cell::new("(error loading)"),
                    comfy_table::Cell::new(""),
                ]);
            }
        }
    }

    print_table(&table);
    println!();

    Ok(())
}
