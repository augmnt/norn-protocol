use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_amount_with_symbol, style_bold, style_dim, truncate_hex_string,
};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_green, cell_yellow, data_table, print_table};

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let info = rpc.get_validator_set().await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} — Epoch {}",
        style_bold().apply_to("Validator Set"),
        info.epoch
    );

    if info.validators.is_empty() {
        println!("  {}", style_dim().apply_to("No validators registered."));
    } else {
        let mut table = data_table(&["Address", "Stake", "Status"]);

        for v in &info.validators {
            let addr_display = truncate_hex_string(&format!("0x{}", v.address), 6);
            let stake: u128 = v.stake.parse().unwrap_or(0);
            let status_cell = if v.active {
                cell_green("\u{25cf} active")
            } else {
                cell_yellow("\u{25cf} inactive")
            };

            table.add_row(vec![
                cell(addr_display),
                cell(format_amount_with_symbol(stake, &NATIVE_TOKEN_ID)),
                status_cell,
            ]);
        }

        print_table(&table);
    }

    let total: u128 = info.total_stake.parse().unwrap_or(0);
    println!(
        "  Total stake: {}",
        format_amount_with_symbol(total, &NATIVE_TOKEN_ID)
    );
    println!();

    Ok(())
}
