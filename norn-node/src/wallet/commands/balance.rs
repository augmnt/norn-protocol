use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount_with_symbol, parse_address, parse_token_id, style_bold,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(
    address: Option<&str>,
    token: Option<&str>,
    json: bool,
) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let rpc = RpcClient::new(&config.rpc_url)?;

    let addr = if let Some(a) = address {
        parse_address(a)?
    } else {
        let name = config.active_wallet_name()?;
        let ks = Keystore::load(name)?;
        ks.address
    };

    let token_id = match token {
        Some(t) => parse_token_id(t)?,
        None => NATIVE_TOKEN_ID,
    };

    let addr_hex = hex::encode(addr);
    let token_hex = hex::encode(token_id);

    let balance_str = rpc.get_balance(&addr_hex, &token_hex).await?;
    let balance: u128 = balance_str
        .parse()
        .map_err(|_| WalletError::RpcError(format!("invalid balance: {}", balance_str)))?;

    if json {
        let info = serde_json::json!({
            "address": format_address(&addr),
            "token_id": hex::encode(token_id),
            "balance": balance_str,
            "human_readable": format_amount_with_symbol(balance, &token_id),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!(
        "  {}: {}",
        style_bold().apply_to("Address"),
        format_address(&addr)
    );
    println!(
        "  {}: {}",
        style_bold().apply_to("Balance"),
        format_amount_with_symbol(balance, &token_id)
    );
    if balance == 0 {
        println!(
            "  {}",
            console::Style::new()
                .dim()
                .apply_to("Hint: Use `norn-node wallet faucet` to get testnet tokens")
        );
    }
    println!();

    Ok(())
}
