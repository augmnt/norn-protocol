use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{format_pubkey, style_bold};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::prompt_password;
use crate::wallet::ui::{cell, cell_cyan, info_table, print_table};

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
    println!("  {}", style_bold().apply_to("Signed Message"));

    let mut table = info_table();
    table.add_row(vec![cell("Message"), cell(message)]);
    table.add_row(vec![cell("Message hash"), cell_cyan(hex::encode(hash))]);
    table.add_row(vec![cell("Signature"), cell_cyan(hex::encode(signature))]);
    table.add_row(vec![
        cell("Public key"),
        cell(format_pubkey(&keypair.public_key())),
    ]);

    print_table(&table);
    println!();

    Ok(())
}
