use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{style_bold, style_dim, truncate_hex_string};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell_right, data_table, print_table};

pub async fn run(limit: u64, json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let tokens = rpc.list_tokens(limit, 0).await?;

    if json {
        let json_str =
            serde_json::to_string_pretty(&tokens).map_err(|e| WalletError::Other(e.to_string()))?;
        println!("{}", json_str);
    } else if tokens.is_empty() {
        println!();
        println!("  No tokens registered yet.");
        println!();
    } else {
        println!();
        println!("  {}", style_bold().apply_to("Registered Tokens"));

        let mut table = data_table(&[
            "Symbol",
            "Name",
            "Supply",
            "Max Supply",
            "Decimals",
            "Creator",
        ]);

        for token in &tokens {
            let max_str = if token.max_supply == "0" {
                "unlimited".to_string()
            } else {
                token.max_supply.clone()
            };
            table.add_row(vec![
                comfy_table::Cell::new(&token.symbol),
                comfy_table::Cell::new(&token.name),
                cell_right(&token.current_supply),
                cell_right(&max_str),
                cell_right(token.decimals),
                comfy_table::Cell::new(truncate_hex_string(&token.creator, 6)),
            ]);
        }

        print_table(&table);
        println!(
            "  {}",
            style_dim().apply_to(format!("{} token(s) found", tokens.len()))
        );
        println!();
    }

    Ok(())
}
