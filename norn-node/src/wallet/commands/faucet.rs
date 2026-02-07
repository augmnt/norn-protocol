use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, parse_address, print_error, print_success};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(address: Option<&str>, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let addr = if let Some(a) = address {
        parse_address(a)?
    } else {
        let name = config.active_wallet_name()?;
        let ks = Keystore::load(name)?;
        ks.address
    };

    println!("  Requesting tokens for {}", format_address(&addr));

    let result = rpc.faucet(&hex::encode(addr)).await?;

    if result.success {
        print_success("Tokens sent! Check your balance with `norn wallet balance`.");
    } else {
        print_error(
            &format!(
                "Faucet request failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }

    Ok(())
}
