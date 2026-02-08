use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, style_bold, style_dim, truncate_hex_string};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_green, cell_yellow, info_table, print_table};

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

    let mut table = info_table();
    table.add_row(vec![cell("Address"), cell(format_address(&ks.address))]);

    let is_registered = thread_info.is_some();
    match thread_info {
        Some(info) => {
            table.add_row(vec![cell("Status"), cell_green("Registered")]);
            table.add_row(vec![cell("Version"), cell(info.version)]);
            table.add_row(vec![
                cell("State"),
                cell(truncate_hex_string(&info.state_hash, 8)),
            ]);
        }
        None => {
            table.add_row(vec![cell("Status"), cell_yellow("Not registered")]);
        }
    }

    print_table(&table);

    if !is_registered {
        println!(
            "  {}",
            style_dim().apply_to("Run `norn wallet register` to register your thread.")
        );
    }
    println!();

    Ok(())
}
