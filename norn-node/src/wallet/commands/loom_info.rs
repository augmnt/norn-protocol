use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, style_bold};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_bold, cell_dim, info_table, print_table};

pub async fn run(loom_id: &str, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let loom_info = match rpc.get_loom_info(loom_id).await? {
        Some(info) => info,
        None => {
            print_error(&format!("loom '{}' not found", loom_id), None);
            return Ok(());
        }
    };

    if json {
        let json_str = serde_json::to_string_pretty(&loom_info)
            .map_err(|e| WalletError::Other(e.to_string()))?;
        println!("{}", json_str);
    } else {
        println!();
        println!("  {}", style_bold().apply_to("Loom Info"));

        let mut table = info_table();

        table.add_row(vec![cell("Name"), cell_bold(&loom_info.name)]);
        table.add_row(vec![
            cell("Active"),
            cell(if loom_info.active { "yes" } else { "no" }),
        ]);
        table.add_row(vec![cell("Operator"), cell(&loom_info.operator)]);
        table.add_row(vec![
            cell("Deployed At"),
            cell(format_timestamp(loom_info.deployed_at)),
        ]);
        table.add_row(vec![cell("Loom ID"), cell_dim(&loom_info.loom_id)]);

        print_table(&table);
        println!();
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
