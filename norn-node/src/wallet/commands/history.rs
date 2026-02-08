use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, style_dim, truncate_hex_string};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_green, cell_yellow, data_table, print_table};

pub async fn run(limit: usize, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;
    let addr_hex = hex::encode(ks.address);

    let entries = rpc
        .get_transaction_history(&addr_hex, limit as u64, 0)
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    if entries.is_empty() {
        println!(
            "  {}",
            style_dim().apply_to("No transaction history found.")
        );
        println!(
            "  {}",
            style_dim().apply_to("Use `norn wallet faucet` to get testnet tokens.")
        );
        println!();
        return Ok(());
    }

    println!(
        "  {} ({})",
        style_bold().apply_to("Transaction History"),
        entries.len()
    );

    let mut table = data_table(&["Time", "Dir", "Amount", "Counterparty", "Memo"]);

    for entry in &entries {
        let time_str = format_timestamp(entry.timestamp);

        let dir_cell = if entry.direction == "sent" {
            cell_yellow("SENT")
        } else {
            cell_green("RCVD")
        };

        let counterparty = if entry.direction == "sent" {
            &entry.to
        } else {
            &entry.from
        };

        let memo = entry
            .memo
            .as_deref()
            .unwrap_or("\u{2014}") // em dash
            .to_string();

        table.add_row(vec![
            cell(&time_str),
            dir_cell,
            cell(&entry.human_readable),
            cell(truncate_hex_string(counterparty, 6)),
            cell(memo),
        ]);
    }

    print_table(&table);
    println!();

    Ok(())
}

/// Format a UNIX timestamp into a human-readable date string.
fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "pending".to_string();
    }
    let dt = chrono::DateTime::from_timestamp(ts as i64, 0);
    match dt {
        Some(d) => d.format("%Y-%m-%d %H:%M").to_string(),
        None => ts.to_string(),
    }
}
