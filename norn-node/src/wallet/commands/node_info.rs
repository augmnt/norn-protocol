use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, style_dim, style_info, style_success, style_warn};
use crate::wallet::rpc_client::RpcClient;

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
    println!("  Status:       {}", status_styled);
    println!("  Version:      {}", style_info().apply_to(&info.version));
    println!("  Chain ID:     {}", info.chain_id);
    println!("  Network:      {}", info.network);
    println!("  Block height: {}", info.height);
    println!("  Validator:    {}", validator_styled);
    println!("  Threads:      {}", info.thread_count);
    println!();

    Ok(())
}
