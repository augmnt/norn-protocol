use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount_with_symbol, style_bold, style_dim, style_info, style_success,
    style_warn,
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
    let token_hex = hex::encode(NATIVE_TOKEN_ID);

    let balance_str = rpc.get_balance(&addr_hex, &token_hex).await?;
    let balance: u128 = balance_str.parse().unwrap_or(0);

    let names = rpc.list_names(&addr_hex).await?;
    let thread_info = rpc.get_thread(&addr_hex).await?;

    if json {
        let info = serde_json::json!({
            "wallet": wallet_name,
            "address": format_address(&ks.address),
            "balance": balance_str,
            "human_readable": format_amount_with_symbol(balance, &NATIVE_TOKEN_ID),
            "names": names.iter().map(|n| &n.name).collect::<Vec<_>>(),
            "thread_registered": thread_info.is_some(),
        });
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
    println!("  Address: {}", format_address(&ks.address));
    println!(
        "  Balance: {}",
        format_amount_with_symbol(balance, &NATIVE_TOKEN_ID)
    );

    if names.is_empty() {
        println!("  Names:   {}", style_dim().apply_to("none"));
    } else {
        let name_list: Vec<&str> = names.iter().map(|n| n.name.as_str()).collect();
        println!("  Names:   {}", style_info().apply_to(name_list.join(", ")));
    }

    match thread_info {
        Some(_) => println!("  Thread:  {}", style_success().apply_to("registered")),
        None => println!("  Thread:  {}", style_warn().apply_to("not registered")),
    }
    println!();

    Ok(())
}
