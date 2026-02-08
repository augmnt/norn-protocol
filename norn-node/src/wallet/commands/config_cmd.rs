use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_success, style_bold};
use crate::wallet::ui::{cell, info_table, print_table};

pub fn run(rpc_url: Option<&str>, network: Option<&str>, json: bool) -> Result<(), WalletError> {
    let mut config = WalletConfig::load()?;

    if let Some(url) = rpc_url {
        config.rpc_url = url.to_string();
        config.save()?;
        print_success(&format!("RPC URL set to {}", url));
        return Ok(());
    }

    if let Some(net) = network {
        match net {
            "dev" | "testnet" | "mainnet" => {
                config.network = net.to_string();
                config.save()?;
                print_success(&format!("Network set to {}", net));
            }
            _ => {
                return Err(WalletError::ConfigError(format!(
                    "unknown network '{}', expected 'dev', 'testnet', or 'mainnet'",
                    net
                )));
            }
        }
        return Ok(());
    }

    // Show current config
    if json {
        let info = serde_json::json!({
            "active_wallet": config.active_wallet,
            "rpc_url": config.rpc_url,
            "network": config.network,
            "wallets": config.wallets,
            "data_dir": WalletConfig::data_dir()?.to_string_lossy(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
        return Ok(());
    }

    println!();
    println!("  {}", style_bold().apply_to("Wallet Configuration"));

    let mut table = info_table();
    table.add_row(vec![
        cell("Active wallet"),
        cell(config.active_wallet.as_deref().unwrap_or("(none)")),
    ]);
    table.add_row(vec![cell("RPC URL"), cell(&config.rpc_url)]);
    table.add_row(vec![cell("Network"), cell(&config.network)]);
    table.add_row(vec![
        cell("Data dir"),
        cell(WalletConfig::data_dir()?.display()),
    ]);
    table.add_row(vec![cell("Wallets"), cell(config.wallets.len())]);

    print_table(&table);
    println!();

    Ok(())
}
