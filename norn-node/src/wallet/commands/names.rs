use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, style_bold, style_dim, style_info};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;
    let addr_hex = hex::encode(ks.address);

    let names = rpc.list_names(&addr_hex).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&names)?);
    } else {
        println!();
        println!(
            "  {} for {}",
            style_bold().apply_to("Registered Names"),
            style_info().apply_to(format_address(&ks.address))
        );
        println!();

        if names.is_empty() {
            println!(
                "  {}",
                style_dim().apply_to("No names registered. Use `norn wallet register-name --name <name>` to register one.")
            );
        } else {
            for name_info in &names {
                println!("  - {}", style_info().apply_to(&name_info.name));
            }
        }
        println!();
    }

    Ok(())
}
