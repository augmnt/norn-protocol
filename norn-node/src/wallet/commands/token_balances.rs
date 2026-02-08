use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount, format_amount_with_symbol, style_bold, style_dim, style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let addr_hex = hex::encode(ks.address);

    // Native NORN balance.
    let native_hex = hex::encode(NATIVE_TOKEN_ID);
    let native_str = rpc.get_balance(&addr_hex, &native_hex).await?;
    let native_bal: u128 = native_str.parse().unwrap_or(0);

    // Block height for context.
    let block_height = rpc
        .get_latest_block()
        .await
        .ok()
        .flatten()
        .map(|b| b.height);

    // Custom token balances.
    let mut holdings: Vec<(String, String, u128)> = Vec::new(); // (symbol, token_id_hex, balance)
    if let Ok(tokens) = rpc.list_tokens(200, 0).await {
        for t in &tokens {
            if let Ok(bal_str) = rpc.get_balance(&addr_hex, &t.token_id).await {
                let bal: u128 = bal_str.parse().unwrap_or(0);
                if bal > 0 {
                    holdings.push((t.symbol.clone(), t.token_id.clone(), bal));
                }
            }
        }
    }

    if json {
        let mut entries = vec![serde_json::json!({
            "symbol": "NORN",
            "token_id": native_hex,
            "balance": native_str,
            "human_readable": format_amount_with_symbol(native_bal, &NATIVE_TOKEN_ID),
        })];
        for (sym, tid, bal) in &holdings {
            entries.push(serde_json::json!({
                "symbol": sym,
                "token_id": tid,
                "balance": bal.to_string(),
                "human_readable": format!("{} {}", format_amount(*bal), sym),
            }));
        }
        let mut info = serde_json::json!({
            "address": format_address(&ks.address),
            "balances": entries,
        });
        if let Some(h) = block_height {
            info["block_height"] = serde_json::json!(h);
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} — {}",
        style_bold().apply_to("Token Balances"),
        format_address(&ks.address)
    );
    if let Some(h) = block_height {
        println!("  {}", style_dim().apply_to(format!("(at block #{})", h)));
    }
    crate::wallet::format::print_divider();

    // NORN first.
    println!(
        "  {} {}",
        style_bold().apply_to(format!("{:>30}", format_amount(native_bal))),
        style_info().apply_to("NORN")
    );

    // Custom tokens.
    for (sym, _tid, bal) in &holdings {
        let formatted = format!("{:>30}", format_amount(*bal));
        println!("  {} {}", formatted, style_info().apply_to(sym));
    }

    if holdings.is_empty() {
        println!("  {}", style_dim().apply_to("No custom token holdings."));
    }
    println!();

    Ok(())
}
