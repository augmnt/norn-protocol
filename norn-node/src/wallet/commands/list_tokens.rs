use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_divider, style_bold, style_dim};
use crate::wallet::rpc_client::RpcClient;

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
        print_divider();

        for token in &tokens {
            let max_str = if token.max_supply == "0" {
                "unlimited".to_string()
            } else {
                token.max_supply.clone()
            };
            println!(
                "  {} {}",
                style_bold().apply_to(format!("{:<12}", token.symbol)),
                token.name,
            );
            println!(
                "    {}",
                style_dim().apply_to(format!(
                    "supply: {} / {}  |  decimals: {}  |  creator: {}",
                    token.current_supply, max_str, token.decimals, token.creator
                ))
            );
            println!(
                "    {}",
                style_dim().apply_to(format!("id: {}", token.token_id))
            );
        }
        println!();
        println!(
            "  {}",
            style_dim().apply_to(format!("{} token(s) found", tokens.len()))
        );
        println!();
    }

    Ok(())
}
