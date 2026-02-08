use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, style_dim, truncate_hex_string};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{data_table, print_table};

pub async fn run(limit: u64, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let looms = rpc.list_looms(limit, 0).await?;

    if json {
        let json_str =
            serde_json::to_string_pretty(&looms).map_err(|e| WalletError::Other(e.to_string()))?;
        println!("{}", json_str);
    } else if looms.is_empty() {
        println!();
        println!("  No looms deployed yet.");
        println!();
    } else {
        println!();
        println!("  {}", style_bold().apply_to("Deployed Looms"));

        let mut table = data_table(&["Name", "Active", "Operator", "Loom ID"]);

        for loom in &looms {
            table.add_row(vec![
                comfy_table::Cell::new(&loom.name),
                comfy_table::Cell::new(if loom.active { "yes" } else { "no" }),
                comfy_table::Cell::new(truncate_hex_string(&loom.operator, 6)),
                comfy_table::Cell::new(truncate_hex_string(&loom.loom_id, 8)),
            ]);
        }

        print_table(&table);
        println!(
            "  {}",
            style_dim().apply_to(format!("{} loom(s) found", looms.len()))
        );
        println!();
    }

    Ok(())
}
