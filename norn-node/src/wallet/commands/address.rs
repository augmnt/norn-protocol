use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, format_pubkey, style_bold};
use crate::wallet::keystore::Keystore;
use crate::wallet::ui::{cell, info_table, print_table};

pub fn run(name: Option<&str>, json: bool) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = match name {
        Some(n) => n,
        None => config.active_wallet_name()?,
    };

    let ks = Keystore::load(wallet_name)?;

    if json {
        let info = serde_json::json!({
            "name": wallet_name,
            "address": format_address(&ks.address),
            "public_key": format_pubkey(&ks.public_key),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {} {}", style_bold().apply_to("Wallet:"), wallet_name);

    let mut table = info_table();
    table.add_row(vec![cell("Address"), cell(format_address(&ks.address))]);
    table.add_row(vec![
        cell("Public key"),
        cell(format_pubkey(&ks.public_key)),
    ]);

    print_table(&table);
    println!();

    Ok(())
}
