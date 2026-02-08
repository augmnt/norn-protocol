use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount, format_amount_with_symbol, style_bold, style_dim, style_info,
    style_success, style_warn,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell_right, data_table, info_table, print_table};

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let addr_hex = hex::encode(ks.address);
    let token_hex = hex::encode(NATIVE_TOKEN_ID);

    let balance_str = rpc.get_balance(&addr_hex, &token_hex).await?;
    let balance: u128 = balance_str.parse().unwrap_or(0);

    let names = rpc.list_names(&addr_hex).await?;
    let thread_info = rpc.get_thread(&addr_hex).await?;

    // Fetch block height.
    let block_height = rpc
        .get_latest_block()
        .await
        .ok()
        .flatten()
        .map(|b| b.height);

    // Fetch custom token balances (non-zero only).
    let mut token_balances: Vec<(String, u128)> = Vec::new();
    if let Ok(tokens) = rpc.list_tokens(200, 0).await {
        for t in &tokens {
            if let Ok(bal_str) = rpc.get_balance(&addr_hex, &t.token_id).await {
                let bal: u128 = bal_str.parse().unwrap_or(0);
                if bal > 0 {
                    token_balances.push((t.symbol.clone(), bal));
                }
            }
        }
    }

    if json {
        let token_holdings: Vec<serde_json::Value> = token_balances
            .iter()
            .map(|(sym, bal)| {
                serde_json::json!({
                    "symbol": sym,
                    "balance": bal.to_string(),
                })
            })
            .collect();

        let mut info = serde_json::json!({
            "wallet": wallet_name,
            "address": format_address(&ks.address),
            "balance": balance_str,
            "human_readable": format_amount_with_symbol(balance, &NATIVE_TOKEN_ID),
            "names": names.iter().map(|n| &n.name).collect::<Vec<_>>(),
            "thread_registered": thread_info.is_some(),
            "token_balances": token_holdings,
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
        "  {} {}",
        style_bold().apply_to("Wallet:"),
        style_info().apply_to(wallet_name)
    );

    let mut table = info_table();

    table.add_row(vec![
        comfy_table::Cell::new("Address"),
        comfy_table::Cell::new(format_address(&ks.address)),
    ]);
    table.add_row(vec![
        comfy_table::Cell::new("Balance"),
        comfy_table::Cell::new(format_amount_with_symbol(balance, &NATIVE_TOKEN_ID)),
    ]);
    if let Some(h) = block_height {
        table.add_row(vec![
            comfy_table::Cell::new("Block"),
            comfy_table::Cell::new(format!("#{}", h)),
        ]);
    }

    let names_display = if names.is_empty() {
        style_dim().apply_to("none").to_string()
    } else {
        let name_list: Vec<&str> = names.iter().map(|n| n.name.as_str()).collect();
        style_info().apply_to(name_list.join(", ")).to_string()
    };
    table.add_row(vec![
        comfy_table::Cell::new("Names"),
        comfy_table::Cell::new(names_display),
    ]);

    let thread_display = match thread_info {
        Some(_) => format!("{} registered", style_success().apply_to("\u{25cf}")),
        None => format!("{} not registered", style_warn().apply_to("\u{25cf}")),
    };
    table.add_row(vec![
        comfy_table::Cell::new("Thread"),
        comfy_table::Cell::new(thread_display),
    ]);

    print_table(&table);

    if !token_balances.is_empty() {
        println!();
        println!("  {}", style_bold().apply_to("Token Holdings"));

        let mut ttable = data_table(&["Token", "Balance"]);
        for (sym, bal) in &token_balances {
            ttable.add_row(vec![
                comfy_table::Cell::new(sym),
                cell_right(format!("{} {}", format_amount(*bal), sym)),
            ]);
        }
        print_table(&ttable);
    }
    println!();

    Ok(())
}
