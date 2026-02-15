use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_amount_with_symbol, style_bold, style_dim, truncate_hex_string,
};
use crate::wallet::rpc_client::RpcClient;
use crate::wallet::ui::{cell, cell_right, data_table, print_table};

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let info = rpc.get_validator_rewards().await?;

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
        style_bold().apply_to("Validator Rewards"),
        info.current_epoch
    );

    let pending: u128 = info.pending_epoch_fees.parse().unwrap_or(0);
    println!(
        "  Pending epoch fees: {}",
        format_amount_with_symbol(pending, &NATIVE_TOKEN_ID)
    );
    println!(
        "  Blocks until distribution: {}",
        info.blocks_until_distribution
    );

    println!();

    if info.projected_rewards.is_empty() {
        println!(
            "  {}",
            style_dim().apply_to("No active validators to receive rewards.")
        );
    } else {
        println!(
            "  {}",
            style_bold().apply_to("Projected Reward Distribution")
        );

        let mut table = data_table(&["Validator", "Stake", "Projected Reward"]);

        for v in &info.projected_rewards {
            let addr_display = truncate_hex_string(&format!("0x{}", v.address), 6);
            let stake: u128 = v.stake.parse().unwrap_or(0);
            let reward: u128 = v.projected_reward.parse().unwrap_or(0);

            table.add_row(vec![
                cell(addr_display),
                cell_right(format_amount_with_symbol(stake, &NATIVE_TOKEN_ID)),
                cell_right(format_amount_with_symbol(reward, &NATIVE_TOKEN_ID)),
            ]);
        }

        print_table(&table);
    }

    println!();

    Ok(())
}
