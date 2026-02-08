use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, truncate_hex_string};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, info_table, print_table};

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let state = rpc.get_weave_state().await?;

    match state {
        Some(info) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&info).unwrap_or_default()
                );
                return Ok(());
            }

            println!();
            println!("  {}", style_bold().apply_to("Weave State"));

            let mut table = info_table();
            table.add_row(vec![cell("Height"), cell(info.height)]);
            table.add_row(vec![
                cell("Latest hash"),
                cell(truncate_hex_string(&info.latest_hash, 8)),
            ]);
            table.add_row(vec![
                cell("Threads root"),
                cell(truncate_hex_string(&info.threads_root, 8)),
            ]);
            table.add_row(vec![cell("Thread count"), cell(info.thread_count)]);
            table.add_row(vec![cell("Base fee"), cell(&info.base_fee)]);
            table.add_row(vec![
                cell("Fee multiplier"),
                cell(format!("{}x", info.fee_multiplier as f64 / 1000.0)),
            ]);

            print_table(&table);
            println!();
        }
        None => {
            println!("  Weave state not available.");
        }
    }

    Ok(())
}
