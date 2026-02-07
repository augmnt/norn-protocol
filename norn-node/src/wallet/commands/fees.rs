use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_amount_with_symbol, style_bold};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let info = rpc.get_fee_estimate().await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    let base_fee: u128 = info
        .base_fee
        .parse()
        .map_err(|_| WalletError::RpcError(format!("invalid base_fee: {}", info.base_fee)))?;
    let fee_per_commitment: u128 = info.fee_per_commitment.parse().map_err(|_| {
        WalletError::RpcError(format!(
            "invalid fee_per_commitment: {}",
            info.fee_per_commitment
        ))
    })?;

    println!();
    println!("  {}", style_bold().apply_to("Fee Estimate"));
    println!(
        "  Base fee:           {}",
        format_amount_with_symbol(base_fee, &NATIVE_TOKEN_ID)
    );
    println!(
        "  Fee multiplier:     {}x",
        info.fee_multiplier as f64 / 1000.0
    );
    println!(
        "  Per commitment:     {}",
        format_amount_with_symbol(fee_per_commitment, &NATIVE_TOKEN_ID)
    );
    println!();

    Ok(())
}
