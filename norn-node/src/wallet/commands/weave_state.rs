use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::style_bold;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(json: bool, rpc_url: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    let state = rpc.get_weave_state().await?;

    match state {
        Some(info) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&info).unwrap_or_default()
                );
                return Ok(());
            }

            println!();
            println!("  {}", style_bold().apply_to("Weave State"));
            println!("  Height:         {}", info.height);
            println!("  Latest hash:    {}", truncate_hash(&info.latest_hash));
            println!("  Threads root:   {}", truncate_hash(&info.threads_root));
            println!("  Thread count:   {}", info.thread_count);
            println!("  Base fee:       {}", info.base_fee);
            println!("  Fee multiplier: {}x", info.fee_multiplier as f64 / 1000.0);
            println!();
        }
        None => {
            println!("  Weave state not available.");
        }
    }

    Ok(())
}

fn truncate_hash(hash: &str) -> String {
    if hash.len() > 16 {
        format!("{}...{}", &hash[..8], &hash[hash.len() - 8..])
    } else {
        hash.to_string()
    }
}
