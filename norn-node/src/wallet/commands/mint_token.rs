use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_token_amount_with_name, parse_token_amount, print_divider, print_error,
    print_success, style_bold, style_info,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};
use crate::wallet::rpc_client::RpcClient;

pub async fn run(
    token: &str,
    to: &str,
    amount: &str,
    yes: bool,
    rpc_url: Option<&str>,
) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = config.active_wallet_name()?;
    let ks = Keystore::load(wallet_name)?;

    let url = rpc_url.unwrap_or(&config.rpc_url);
    let rpc = RpcClient::new(url)?;

    // Resolve token first so we know the correct decimals for amount parsing.
    let token_info = resolve_token(&rpc, token).await?;
    let token_id = hex_to_token_id(&token_info.token_id)?;

    let amount_val = parse_token_amount(amount, token_info.decimals)?;

    if amount_val == 0 {
        return Err(WalletError::Other("amount must be > 0".to_string()));
    }

    // Parse recipient address.
    let to_hex = to.strip_prefix("0x").unwrap_or(to);
    let to_bytes = hex::decode(to_hex)
        .map_err(|_| WalletError::Other("invalid recipient address".to_string()))?;
    if to_bytes.len() != 20 {
        return Err(WalletError::Other(
            "recipient address must be 20 bytes".to_string(),
        ));
    }
    let mut to_addr = [0u8; 20];
    to_addr.copy_from_slice(&to_bytes);

    // Verify caller is the token creator.
    let authority = norn_crypto::address::pubkey_to_address(&ks.public_key);
    let creator_hex = token_info
        .creator
        .strip_prefix("0x")
        .unwrap_or(&token_info.creator);
    let authority_hex = hex::encode(authority);
    if authority_hex != creator_hex {
        print_error(
            &format!(
                "only the token creator ({}) can mint; your address is {}",
                token_info.creator,
                format_address(&authority)
            ),
            None,
        );
        return Ok(());
    }

    // Show confirmation.
    if !yes {
        println!();
        println!("  {}", style_bold().apply_to("Mint Tokens"));
        print_divider();
        println!(
            "  Token:     {} ({})",
            style_info().apply_to(&token_info.symbol),
            &token_info.token_id[..16]
        );
        println!("  To:        {}", format_address(&to_addr));
        println!(
            "  Amount:    {}",
            style_bold().apply_to(format_token_amount_with_name(
                amount_val,
                token_info.decimals,
                &token_info.symbol
            ))
        );
        println!();

        if !confirm("Mint these tokens?")? {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut token_mint = norn_types::weave::TokenMint {
        token_id,
        to: to_addr,
        amount: amount_val,
        authority,
        authority_pubkey: keypair.public_key(),
        timestamp: now,
        signature: [0u8; 64],
    };

    let sig_data = norn_weave::token::token_mint_signing_data(&token_mint);
    token_mint.signature = keypair.sign(&sig_data);

    let bytes =
        borsh::to_vec(&token_mint).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let hex_data = hex::encode(&bytes);

    let result = rpc.mint_token(&hex_data).await?;

    if result.success {
        print_success(&format!(
            "Minted {} to {}",
            format_token_amount_with_name(amount_val, token_info.decimals, &token_info.symbol),
            format_address(&to_addr)
        ));
    } else {
        print_error(
            &format!(
                "Mint failed: {}",
                result.reason.unwrap_or_else(|| "unknown".to_string())
            ),
            None,
        );
    }
    println!();

    Ok(())
}

pub async fn resolve_token(
    rpc: &RpcClient,
    token: &str,
) -> Result<crate::rpc::types::TokenInfo, WalletError> {
    // Try by symbol first (short strings that aren't valid hex).
    if token.len() <= 12 && !token.chars().all(|c| c.is_ascii_hexdigit()) {
        if let Some(info) = rpc.get_token_by_symbol(token).await? {
            return Ok(info);
        }
        return Err(WalletError::Other(format!("token '{}' not found", token)));
    }
    // Try as hex token ID.
    let hex_str = token.strip_prefix("0x").unwrap_or(token);
    if let Some(info) = rpc.get_token_info(hex_str).await? {
        return Ok(info);
    }
    // Fallback: try as symbol.
    if let Some(info) = rpc.get_token_by_symbol(token).await? {
        return Ok(info);
    }
    Err(WalletError::Other(format!("token '{}' not found", token)))
}

pub fn hex_to_token_id(hex_str: &str) -> Result<[u8; 32], WalletError> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes =
        hex::decode(hex_str).map_err(|_| WalletError::Other("invalid token ID hex".to_string()))?;
    if bytes.len() != 32 {
        return Err(WalletError::Other("token ID must be 32 bytes".to_string()));
    }
    let mut id = [0u8; 32];
    id.copy_from_slice(&bytes);
    Ok(id)
}
