use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount_with_token_name, parse_address, style_bold, style_dim,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_dim, info_table, print_table};

pub async fn run(
    address: Option<&str>,
    token: Option<&str>,
    json: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
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

    // Resolve token: handle native shortcuts locally, else RPC symbol lookup.
    let (token_id, token_symbol) = match token {
        Some(t) if t.eq_ignore_ascii_case("norn") || t == "native" => {
            (NATIVE_TOKEN_ID, "NORN".to_string())
        }
        Some(t) => {
            let info = super::mint_token::resolve_token(&rpc, t).await?;
            let id = super::mint_token::hex_to_token_id(&info.token_id)?;
            (id, info.symbol)
        }
        None => (NATIVE_TOKEN_ID, "NORN".to_string()),
    };

    let addr_hex = hex::encode(addr);
    let token_hex = hex::encode(token_id);

    let balance_str = rpc.get_balance(&addr_hex, &token_hex).await?;
    let balance: u128 = balance_str
        .parse()
        .map_err(|_| WalletError::RpcError(format!("invalid balance: {}", balance_str)))?;

    // Fetch block height for context.
    let block_height = rpc
        .get_latest_block()
        .await
        .ok()
        .flatten()
        .map(|b| b.height);

    if json {
        let mut info = serde_json::json!({
            "address": format_address(&addr),
            "token_id": hex::encode(token_id),
            "token_symbol": token_symbol,
            "balance": balance_str,
            "human_readable": format_amount_with_token_name(balance, &token_symbol),
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
    println!("  {}", style_bold().apply_to("Balance"));

    let mut table = info_table();
    table.add_row(vec![cell("Address"), cell(format_address(&addr))]);
    table.add_row(vec![
        cell("Balance"),
        cell(format_amount_with_token_name(balance, &token_symbol)),
    ]);
    if let Some(h) = block_height {
        table.add_row(vec![cell("Block"), cell_dim(format!("#{}", h))]);
    }

    print_table(&table);

    if balance == 0 {
        println!(
            "  {}",
            style_dim().apply_to("Hint: Use `norn wallet faucet` to get testnet tokens")
        );
    }
    println!();

    Ok(())
}
