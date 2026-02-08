use norn_types::constants::MAX_SUPPLY;

use crate::rpc::types::TokenInfo;
use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_amount, print_divider, print_error, style_bold, style_dim, style_info,
};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(token: &str, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Handle native NORN without RPC lookup.
    let is_native = token.eq_ignore_ascii_case("norn") || token == "native";

    let token_info = if is_native {
        // Native NORN supply isn't tracked in the NT-1 registry.
        // Show max supply as reference since per-token supply tracking is for custom tokens.
        let current_supply = if rpc.get_weave_state().await?.is_some() {
            format_amount(MAX_SUPPLY)
        } else {
            "N/A".to_string()
        };

        TokenInfo {
            token_id: hex::encode([0u8; 32]),
            name: "Norn".to_string(),
            symbol: "NORN".to_string(),
            decimals: 12,
            max_supply: MAX_SUPPLY.to_string(),
            current_supply,
            creator: "protocol (native)".to_string(),
            created_at: 0,
        }
    } else {
        // Resolve custom token (by symbol or hex ID).
        match super::mint_token::resolve_token(&rpc, token).await {
            Ok(info) => info,
            Err(_) => {
                print_error(&format!("token '{}' not found", token), None);
                return Ok(());
            }
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
            } else if is_native {
                format_amount(MAX_SUPPLY)
            } else {
                token_info.max_supply.clone()
            }
        );
        println!("  Current Supply: {}", token_info.current_supply);
        println!("  Creator:        {}", token_info.creator);
        if !is_native {
            println!(
                "  Created At:     {}",
                style_dim().apply_to(format_timestamp(token_info.created_at))
            );
        }
        println!(
            "  Token ID:       {}",
            style_dim().apply_to(if is_native {
                "native (0x0000...0000)".to_string()
            } else {
                token_info.token_id.clone()
            })
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
