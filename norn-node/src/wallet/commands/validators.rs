use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_amount_with_symbol, style_bold, style_dim, style_success, style_warn,
};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let info = rpc.get_validator_set().await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {}", style_bold().apply_to("Validator Set"));
    println!("  Epoch: {}", info.epoch);
    println!();

    if info.validators.is_empty() {
        println!("  {}", style_dim().apply_to("No validators registered."));
    } else {
        for v in &info.validators {
            let addr_display = if v.address.len() > 16 {
                format!(
                    "0x{}...{}",
                    &v.address[..6],
                    &v.address[v.address.len() - 4..]
                )
            } else {
                format!("0x{}", v.address)
            };

            let stake: u128 = v.stake.parse().unwrap_or(0);
            let status = if v.active {
                style_success().apply_to("active").to_string()
            } else {
                style_warn().apply_to("inactive").to_string()
            };

            println!(
                "  {}  {}  {}",
                addr_display,
                format_amount_with_symbol(stake, &NATIVE_TOKEN_ID),
                status,
            );
        }
    }

    let total: u128 = info.total_stake.parse().unwrap_or(0);
    println!();
    println!(
        "  Total stake: {}",
        format_amount_with_symbol(total, &NATIVE_TOKEN_ID)
    );
    println!();

    Ok(())
}
