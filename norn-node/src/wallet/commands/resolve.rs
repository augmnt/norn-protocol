use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, style_bold, style_info};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(name: &str, json: bool) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let rpc = RpcClient::new(&config.rpc_url)?;

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
