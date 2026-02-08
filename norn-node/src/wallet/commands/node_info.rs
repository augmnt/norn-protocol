use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::style_bold;
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{
    cell, cell_cyan, cell_dim, cell_green, cell_yellow, info_table, print_table,
};

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let info = rpc.health().await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    let status_cell = if info.status == "ok" {
        cell_green(&info.status)
    } else {
        cell_yellow(&info.status)
    };

    let validator_cell = if info.is_validator {
        cell_green("yes")
    } else {
        cell_dim("no")
    };

    println!();
    println!("  {}", style_bold().apply_to("Node Info"));

    let mut table = info_table();
    table.add_row(vec![cell("Status"), status_cell]);
    table.add_row(vec![cell("Version"), cell_cyan(&info.version)]);
    table.add_row(vec![cell("Chain ID"), cell(&info.chain_id)]);
    table.add_row(vec![cell("Network"), cell(&info.network)]);
    table.add_row(vec![cell("Block height"), cell(info.height)]);
    table.add_row(vec![cell("Validator"), validator_cell]);
    table.add_row(vec![cell("Threads"), cell(info.thread_count)]);

    print_table(&table);
    println!();

    Ok(())
}
