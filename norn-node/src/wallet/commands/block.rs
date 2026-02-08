use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, truncate_hex_string};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{info_table, print_table};

pub async fn run(
    height: Option<&str>,
    json: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let block = if let Some(h) = height {
        if h == "latest" {
            rpc.get_latest_block().await?
        } else {
            let height: u64 = h
                .parse()
                .map_err(|_| WalletError::Other(format!("invalid height: {}", h)))?;
            rpc.get_block(height).await?
        }
    } else {
        rpc.get_latest_block().await?
    };

    match block {
        Some(info) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&info).unwrap_or_default()
                );
                return Ok(());
            }

            let proposer_display = if info.proposer.is_empty() {
                "(none)".to_string()
            } else {
                truncate_hex_string(&info.proposer, 8)
            };

            println!();
            println!(
                "  {}",
                style_bold().apply_to(format!("Block #{}", info.height))
            );

            let mut table = info_table();
            table.add_row(vec!["Hash", &info.hash]);
            table.add_row(vec!["Prev hash", &info.prev_hash]);
            table.add_row(vec!["Timestamp", &format_timestamp(info.timestamp)]);
            table.add_row(vec!["Proposer", &proposer_display]);
            table.add_row(vec!["Commitments", &info.commitment_count.to_string()]);
            table.add_row(vec!["Registrations", &info.registration_count.to_string()]);
            table.add_row(vec!["Anchors", &info.anchor_count.to_string()]);
            table.add_row(vec!["Fraud proofs", &info.fraud_proof_count.to_string()]);
            table.add_row(vec![
                "Name registrations",
                &info.name_registration_count.to_string(),
            ]);
            table.add_row(vec!["Transfers", &info.transfer_count.to_string()]);
            table.add_row(vec![
                "Token definitions",
                &info.token_definition_count.to_string(),
            ]);
            table.add_row(vec!["Token mints", &info.token_mint_count.to_string()]);
            table.add_row(vec!["Token burns", &info.token_burn_count.to_string()]);

            print_table(&table);
            println!();
        }
        None => {
            println!("  Block not found.");
        }
    }

    Ok(())
}

fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "genesis".to_string();
    }
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}
