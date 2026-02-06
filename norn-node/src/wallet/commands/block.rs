use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::style_bold;
use crate::wallet::rpc_client::RpcClient;

pub async fn run(height: Option<&str>, json: bool) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let rpc = RpcClient::new(&config.rpc_url)?;

    let block = if let Some(h) = height {
        if h == "latest" {
            rpc.get_latest_block().await?
        } else {
            let height: u64 = h
                .parse()
                .map_err(|_| WalletError::Other(format!("invalid height: {}", h)))?;
            rpc.get_block(height).await?
        }
    } else {
        rpc.get_latest_block().await?
    };

    match block {
        Some(info) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&info).unwrap_or_default()
                );
                return Ok(());
            }

            println!();
            println!(
                "  {}",
                style_bold().apply_to(format!("Block #{}", info.height))
            );
            println!("  Hash:           {}", info.hash);
            println!("  Prev hash:      {}", info.prev_hash);
            println!("  Timestamp:      {}", format_timestamp(info.timestamp));
            println!("  Proposer:       {}", truncate_or_empty(&info.proposer));
            println!("  Commitments:    {}", info.commitment_count);
            println!("  Registrations:  {}", info.registration_count);
            println!("  Anchors:        {}", info.anchor_count);
            println!("  Fraud proofs:   {}", info.fraud_proof_count);
            println!();
        }
        None => {
            println!("  Block not found.");
        }
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

fn truncate_or_empty(s: &str) -> String {
    if s.is_empty() {
        "(none)".to_string()
    } else if s.len() > 16 {
        format!("{}...{}", &s[..8], &s[s.len() - 8..])
    } else {
        s.to_string()
    }
}
