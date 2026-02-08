use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_pubkey, style_bold, style_info};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::prompt_password;

pub fn run(message: &str, name: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = match name {
        Some(n) => n,
        None => config.active_wallet_name()?,
    };

    let ks = Keystore::load(wallet_name)?;
    let password = prompt_password("Enter password")?;
    let keypair = ks.decrypt_keypair(&password)?;

    // Hash the message with BLAKE3
    let hash = norn_crypto::hash::blake3_hash(message.as_bytes());

    // Sign the hash
    let signature = keypair.sign(&hash);

    println!();
    println!("  {}: {}", style_bold().apply_to("Message"), message);
    println!(
        "  {}: {}",
        style_bold().apply_to("Message hash"),
        style_info().apply_to(hex::encode(hash))
    );
    println!(
        "  {}: {}",
        style_bold().apply_to("Signature"),
        style_info().apply_to(hex::encode(signature))
    );
    println!(
        "  {}: {}",
        style_bold().apply_to("Public key"),
        format_pubkey(&keypair.public_key())
    );
    println!();

    Ok(())
}
