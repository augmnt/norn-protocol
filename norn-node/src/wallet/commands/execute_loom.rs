use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, print_success, style_dim};
use crate::wallet::keystore::Keystore;
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_bold, info_table, print_table};

pub async fn run(loom_id: &str, input_hex: &str, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let sender_hex = hex::encode(ks.address);

    // Validate input is valid hex.
    hex::decode(input_hex).map_err(|e| WalletError::Other(format!("invalid input hex: {}", e)))?;

    let result = rpc.execute_loom(loom_id, input_hex, &sender_hex).await?;

    println!();
    if result.success {
        print_success("Loom execution succeeded");

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
                "Loom execution failed: {}",
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
