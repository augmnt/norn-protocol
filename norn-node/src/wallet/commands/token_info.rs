use norn_types::constants::MAX_SUPPLY;

use crate::rpc::types::TokenInfo;
use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_amount, print_error, style_bold};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_bold, cell_dim, info_table, print_table};

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

        let mut table = info_table();

        table.add_row(vec![cell("Name"), cell(&token_info.name)]);
        table.add_row(vec![cell("Symbol"), cell_bold(&token_info.symbol)]);
        table.add_row(vec![cell("Decimals"), cell(token_info.decimals)]);

        let max_display = if token_info.max_supply == "0" {
            "unlimited".to_string()
        } else if is_native {
            format_amount(MAX_SUPPLY)
        } else {
            token_info.max_supply.clone()
        };
        table.add_row(vec![cell("Max Supply"), cell(&max_display)]);
        table.add_row(vec![
            cell("Current Supply"),
            cell(&token_info.current_supply),
        ]);
        table.add_row(vec![cell("Creator"), cell(&token_info.creator)]);

        if !is_native {
            table.add_row(vec![
                cell("Created At"),
                cell(format_timestamp(token_info.created_at)),
            ]);
        }

        let id_display = if is_native {
            "native (0x0000...0000)".to_string()
        } else {
            token_info.token_id.clone()
        };
        table.add_row(vec![cell("Token ID"), cell_dim(id_display)]);

        print_table(&table);
        println!();
    }

    Ok(())
}

fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "genesis".to_string();
    }
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}
