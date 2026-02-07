use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, style_dim, style_info, style_success, style_warn};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;

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
            style_dim().apply_to("Use `norn-node wallet faucet` to get testnet tokens.")
        );
        println!();
        return Ok(());
    }

    println!(
        "  {} ({})",
        style_bold().apply_to("Transaction History"),
        entries.len()
    );
    println!(
        "  {}",
        style_dim().apply_to("────────────────────────────────────────────────────────────────")
    );

    for entry in &entries {
        let direction_style = if entry.direction == "sent" {
            style_warn()
        } else {
            style_success()
        };
        let arrow = if entry.direction == "sent" {
            "->"
        } else {
            "<-"
        };

        // Format timestamp as relative or absolute.
        let time_str = format_timestamp(entry.timestamp);

        // Truncate knot_id for display.
        let knot_short = if entry.knot_id.len() > 16 {
            format!(
                "{}...{}",
                &entry.knot_id[..8],
                &entry.knot_id[entry.knot_id.len() - 8..]
            )
        } else {
            entry.knot_id.clone()
        };

        let counterparty = if entry.direction == "sent" {
            &entry.to
        } else {
            &entry.from
        };

        println!(
            "  {} {} {} {} {}",
            style_dim().apply_to(&time_str),
            direction_style.apply_to(format!("{:>8}", entry.direction.to_uppercase())),
            style_bold().apply_to(&entry.human_readable),
            arrow,
            style_info().apply_to(counterparty)
        );

        if let Some(ref memo) = entry.memo {
            println!("           {} \"{}\"", style_dim().apply_to("memo:"), memo);
        }
        println!(
            "           {} {}",
            style_dim().apply_to("knot:"),
            style_dim().apply_to(&knot_short)
        );
    }

    println!();

    Ok(())
}

/// Format a UNIX timestamp into a human-readable date string.
fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "pending".to_string();
    }
    // Use chrono for formatting if available.
    let dt = chrono::DateTime::from_timestamp(ts as i64, 0);
    match dt {
        Some(d) => d.format("%Y-%m-%d %H:%M").to_string(),
        None => ts.to_string(),
    }
}
