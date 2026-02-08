use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount, format_amount_with_symbol, format_token_amount, style_bold,
    style_dim,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell_right, data_table, print_table};

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
    let mut holdings: Vec<(String, String, u128, u8)> = Vec::new(); // (symbol, token_id_hex, balance, decimals)
    if let Ok(tokens) = rpc.list_tokens(200, 0).await {
        for t in &tokens {
            if let Ok(bal_str) = rpc.get_balance(&addr_hex, &t.token_id).await {
                let bal: u128 = bal_str.parse().unwrap_or(0);
                if bal > 0 {
                    holdings.push((t.symbol.clone(), t.token_id.clone(), bal, t.decimals));
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
        for (sym, tid, bal, decimals) in &holdings {
            entries.push(serde_json::json!({
                "symbol": sym,
                "token_id": tid,
                "balance": bal.to_string(),
                "human_readable": format!("{} {}", format_token_amount(*bal, *decimals), sym),
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

    let height_suffix = block_height
        .map(|h| format!(" (at block #{})", h))
        .unwrap_or_default();
    println!();
    println!(
        "  {} — {}{}",
        style_bold().apply_to("Token Balances"),
        format_address(&ks.address),
        style_dim().apply_to(&height_suffix),
    );

    let mut table = data_table(&["Token", "Balance"]);

    // NORN first.
    table.add_row(vec![
        comfy_table::Cell::new("NORN"),
        cell_right(format!("{} NORN", format_amount(native_bal))),
    ]);

    // Custom tokens.
    for (sym, _tid, bal, decimals) in &holdings {
        table.add_row(vec![
            comfy_table::Cell::new(sym),
            cell_right(format!("{} {}", format_token_amount(*bal, *decimals), sym)),
        ]);
    }

    print_table(&table);

    if holdings.is_empty() {
        println!("  {}", style_dim().apply_to("No custom token holdings."));
    }
    println!();

    Ok(())
}
