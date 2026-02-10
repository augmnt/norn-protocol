use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_divider, style_bold, style_dim, style_info};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(validator_hex: Option<&str>, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let info = rpc.get_staking_info(validator_hex).await?;

    println!();
    println!("  {}", style_bold().apply_to("Staking Information"));
    print_divider();
    println!(
        "  Total Staked:    {}",
        style_info().apply_to(&info.total_staked)
    );
    println!(
        "  Min Stake:       {}",
        style_dim().apply_to(&info.min_stake)
    );
    println!(
        "  Bonding Period:  {} blocks",
        style_dim().apply_to(info.bonding_period)
    );
    println!(
        "  Validators:      {}",
        style_info().apply_to(info.validators.len())
    );
    println!();

    if info.validators.is_empty() {
        println!("  No active validators.");
    } else {
        for (i, v) in info.validators.iter().enumerate() {
            println!(
                "  {}. {} {}",
                i + 1,
                style_bold().apply_to(&v.pubkey[..16]),
                if v.active {
                    style_info().apply_to("active")
                } else {
                    style_dim().apply_to("inactive")
                }
            );
            println!("     Address: {}", &v.address[..16]);
            println!("     Stake:   {}", v.stake);
            println!();
        }
    }

    Ok(())
}
