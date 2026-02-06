use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_amount_with_symbol, parse_address, parse_amount, parse_token_id,
    print_divider, print_error, print_success, style_bold, style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(
    to: &str,
    amount_str: &str,
    token: Option<&str>,
    memo: Option<&str>,
    yes: bool,
) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let to_addr = parse_address(to)?;
    let amount = parse_amount(amount_str)?;
    let token_id = match token {
        Some(t) => parse_token_id(t)?,
        None => NATIVE_TOKEN_ID,
    };

    if amount == 0 {
        return Err(WalletError::InvalidAmount(
            "amount must be greater than zero".to_string(),
        ));
    }

    // Show confirmation
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Transfer Summary"));
        print_divider();
        println!(
            "  From:    {} ({})",
            format_address(&ks.address),
            wallet_name
        );
        println!(
            "  To:      {}",
            style_info().apply_to(format_address(&to_addr))
        );
        println!(
            "  Amount:  {}",
            style_bold().apply_to(format_amount_with_symbol(amount, &token_id))
        );
        if let Some(m) = memo {
            println!("  Memo:    \"{}\"", m);
        }
        println!();

        if !confirm("Confirm transfer?")? {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;

    let sender_addr = norn_crypto::address::pubkey_to_address(&keypair.public_key());

    // Build the transfer knot
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // For a local dev/solo transfer, we build and sign the knot with the sender's key.
    // The node will need to handle state lookup. We construct a minimal signed knot
    // and submit it. A full implementation would query thread states from the node first.

    let memo_bytes = memo.map(|m| m.as_bytes().to_vec());
    let payload = norn_types::knot::KnotPayload::Transfer(norn_types::knot::TransferPayload {
        token_id,
        amount,
        from: sender_addr,
        to: to_addr,
        memo: memo_bytes,
    });

    // Build knot with minimal states (the node validates actual state)
    let sender_state = norn_types::thread::ThreadState::new();
    let receiver_state = norn_types::thread::ThreadState::new();

    let knot = norn_thread::knot::KnotBuilder::transfer(now)
        .add_before_state(sender_addr, keypair.public_key(), 0, &sender_state)
        .add_before_state(to_addr, [0u8; 32], 0, &receiver_state)
        .add_after_state(sender_addr, keypair.public_key(), 1, &sender_state)
        .add_after_state(to_addr, [0u8; 32], 1, &receiver_state)
        .with_payload(payload)
        .build()?;

    let sig = norn_thread::knot::sign_knot(&knot, &keypair);
    let mut signed_knot = knot;
    norn_thread::knot::add_signature(&mut signed_knot, sig);

    // Serialize and submit
    let bytes =
        borsh::to_vec(&signed_knot).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let hex_data = hex::encode(&bytes);

    let rpc = RpcClient::new(&config.rpc_url)?;
    let result = rpc.submit_knot(&hex_data).await?;

    if result.success {
        print_success(&format!(
            "Transfer of {} sent!",
            format_amount_with_symbol(amount, &token_id)
        ));
        println!(
            "  Knot ID: {}",
            style_info().apply_to(hex::encode(signed_knot.id))
        );
    } else {
        print_error(
            &format!(
                "Transfer failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            Some("Ensure your thread is registered and has sufficient balance."),
        );
    }
    println!();

    Ok(())
}
