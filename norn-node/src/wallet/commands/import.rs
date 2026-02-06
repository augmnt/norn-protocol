use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_address, format_pubkey, print_success, style_bold};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::prompt_new_password;

use dialoguer::Password;

pub fn run(
    use_mnemonic: bool,
    private_key: Option<&str>,
    name: &str,
    passphrase: Option<&str>,
) -> Result<(), WalletError> {
    let mut config = WalletConfig::load()?;
    if config.wallets.contains(&name.to_string()) {
        return Err(WalletError::WalletAlreadyExists(name.to_string()));
    }

    let password = prompt_new_password()?;

    let ks = if use_mnemonic {
        // Prompt for mnemonic
        let phrase = Password::new()
            .with_prompt("Enter mnemonic phrase (24 words)")
            .interact()
            .map_err(|e| WalletError::IoError(std::io::Error::other(e)))?;
        let mnemonic = norn_crypto::seed::parse_mnemonic(&phrase)?;
        Keystore::create(name, &mnemonic, passphrase.unwrap_or(""), &password)?
    } else if let Some(pk_hex) = private_key {
        let hex_str = pk_hex.strip_prefix("0x").unwrap_or(pk_hex);
        let bytes = hex::decode(hex_str)
            .map_err(|e| WalletError::InvalidAddress(format!("invalid private key hex: {}", e)))?;
        if bytes.len() != 32 {
            return Err(WalletError::InvalidAddress(format!(
                "private key must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes);
        Keystore::from_private_key(name, &seed, &password)?
    } else {
        return Err(WalletError::Other(
            "specify --mnemonic or --private-key".to_string(),
        ));
    };

    ks.save()?;

    config.add_wallet(name);
    if config.active_wallet.is_none() {
        config.active_wallet = Some(name.to_string());
    }
    config.save()?;

    println!();
    println!(
        "  {} {}",
        style_bold().apply_to("Wallet imported:"),
        style_bold().apply_to(name)
    );
    println!("  Address:    {}", format_address(&ks.address));
    println!("  Public key: {}", format_pubkey(&ks.public_key));
    print_success("Wallet saved and encrypted.");

    Ok(())
}
