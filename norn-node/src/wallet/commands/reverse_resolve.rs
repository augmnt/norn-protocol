use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, style_bold, style_info};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(address_hex: &str, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    match rpc.reverse_name(address_hex).await? {
        Some(name) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "address": address_hex, "name": name })
                );
            } else {
                println!();
                println!(
                    "  {} -> {}",
                    style_bold().apply_to(address_hex),
                    style_info().apply_to(&name)
                );
                println!();
            }
        }
        None => {
            if json {
                println!("null");
            } else {
                print_error(
                    &format!("no name found for address {}", address_hex),
                    Some("Register a name with `norn wallet register-name --name <name>`"),
                );
            }
        }
    }

    Ok(())
}
