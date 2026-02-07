use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, style_bold, style_info};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(name: &str, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    match rpc.resolve_name(name).await? {
        Some(resolution) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&resolution)?);
            } else {
                println!();
                println!(
                    "  {} -> {}",
                    style_bold().apply_to(name),
                    style_info().apply_to(&resolution.owner)
                );
                println!();
            }
        }
        None => {
            if json {
                println!("null");
            } else {
                print_error(&format!("name '{}' is not registered", name), None);
            }
        }
    }

    Ok(())
}
