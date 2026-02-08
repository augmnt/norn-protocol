use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_divider, print_error, style_bold, style_dim, style_info};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(token: &str, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Resolve token (by symbol or hex ID).
    let token_info = match super::mint_token::resolve_token(&rpc, token).await {
        Ok(info) => info,
        Err(_) => {
            print_error(&format!("token '{}' not found", token), None);
            return Ok(());
        }
    };

    if json {
        let json_str = serde_json::to_string_pretty(&token_info)
            .map_err(|e| WalletError::Other(e.to_string()))?;
        println!("{}", json_str);
    } else {
        println!();
        println!("  {}", style_bold().apply_to("Token Info"));
        print_divider();
        println!(
            "  Name:           {}",
            style_info().apply_to(&token_info.name)
        );
        println!(
            "  Symbol:         {}",
            style_bold().apply_to(&token_info.symbol)
        );
        println!("  Decimals:       {}", token_info.decimals);
        println!(
            "  Max Supply:     {}",
            if token_info.max_supply == "0" {
                "unlimited".to_string()
            } else {
                token_info.max_supply.clone()
            }
        );
        println!("  Current Supply: {}", token_info.current_supply);
        println!("  Creator:        {}", token_info.creator);
        println!(
            "  Created At:     {}",
            style_dim().apply_to(format_timestamp(token_info.created_at))
        );
        println!(
            "  Token ID:       {}",
            style_dim().apply_to(&token_info.token_id)
        );
        println!();
    }

    Ok(())
}

fn format_timestamp(ts: u64) -> String {
    // Simple human-readable timestamp.
    let secs = ts;
    let days = secs / 86400;
    if days > 365 {
        format!("{} ({}d ago)", ts, days)
    } else {
        ts.to_string()
    }
}
