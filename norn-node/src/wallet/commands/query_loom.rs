use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, print_success, style_dim};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_bold, info_table, print_table};

pub async fn run(
    loom_id: &str,
    input_hex: Option<&str>,
    json: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let input = input_hex.unwrap_or("");

    // Validate input is valid hex (if provided).
    if !input.is_empty() {
        hex::decode(input).map_err(|e| WalletError::Other(format!("invalid input hex: {}", e)))?;
    }

    let result = rpc.query_loom(loom_id, input).await?;

    if json {
        let json_str =
            serde_json::to_string_pretty(&result).map_err(|e| WalletError::Other(e.to_string()))?;
        println!("{}", json_str);
        return Ok(());
    }

    println!();
    if result.success {
        print_success("Loom query succeeded");

        let mut table = info_table();

        if let Some(ref output) = result.output_hex {
            table.add_row(vec![cell("Output"), cell_bold(output)]);
        }
        table.add_row(vec![cell("Gas Used"), cell(result.gas_used.to_string())]);

        if !result.logs.is_empty() {
            table.add_row(vec![cell("Logs"), cell(result.logs.join("\n"))]);
        }

        print_table(&table);
    } else {
        print_error(
            &format!(
                "Loom query failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
        println!(
            "  {}",
            style_dim().apply_to(format!("Gas used: {}", result.gas_used))
        );
    }
    println!();

    Ok(())
}
