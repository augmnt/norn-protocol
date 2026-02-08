use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, style_dim, style_info, style_success, style_warn};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{info_table, print_table};

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

    let status_styled = if info.status == "ok" {
        style_success().apply_to(&info.status).to_string()
    } else {
        style_warn().apply_to(&info.status).to_string()
    };

    let validator_styled = if info.is_validator {
        style_success().apply_to("yes").to_string()
    } else {
        style_dim().apply_to("no").to_string()
    };

    println!();
    println!("  {}", style_bold().apply_to("Node Info"));

    let mut table = info_table();
    table.add_row(vec!["Status", &status_styled]);
    table.add_row(vec![
        "Version",
        &style_info().apply_to(&info.version).to_string(),
    ]);
    table.add_row(vec!["Chain ID", &info.chain_id]);
    table.add_row(vec!["Network", &info.network]);
    table.add_row(vec!["Block height", &info.height.to_string()]);
    table.add_row(vec!["Validator", &validator_styled]);
    table.add_row(vec!["Threads", &info.thread_count.to_string()]);

    print_table(&table);
    println!();

    Ok(())
}
