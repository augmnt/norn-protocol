use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{
    format_address, format_pubkey, print_mnemonic_box, print_success, style_bold,
};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::prompt_new_password;

pub fn run(name: &str, passphrase: Option<&str>) -> Result<(), WalletError> {
    // Check if wallet already exists
    let mut config = WalletConfig::load()?;
    if config.wallets.contains(&name.to_string()) {
        return Err(WalletError::WalletAlreadyExists(name.to_string()));
    }

    // Generate mnemonic
    let mnemonic = norn_crypto::seed::generate_mnemonic();
    let phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();

    // Prompt for password
    let password = prompt_new_password()?;

    // Create keystore
    let ks = Keystore::create(name, &mnemonic, passphrase.unwrap_or(""), &password)?;
    ks.save()?;

    // Update config
    config.add_wallet(name);
    if config.active_wallet.is_none() {
        config.active_wallet = Some(name.to_string());
    }
    config.save()?;

    // Display results
    println!();
    println!(
        "  {} {}",
        style_bold().apply_to("Wallet created:"),
        style_bold().apply_to(name)
    );
    println!("  Address:    {}", format_address(&ks.address));
    println!("  Public key: {}", format_pubkey(&ks.public_key));

    print_mnemonic_box(&words);
    print_success("Wallet saved and encrypted.");

    Ok(())
}
