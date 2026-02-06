use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, style_bold, style_warn};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{confirm, prompt_password};

pub fn run(
    name: Option<&str>,
    show_mnemonic: bool,
    show_private_key: bool,
) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = match name {
        Some(n) => n,
        None => config.active_wallet_name()?,
    };

    let ks = Keystore::load(wallet_name)?;

    if !show_mnemonic && !show_private_key {
        print_error(
            "specify --show-mnemonic or --show-private-key",
            Some("norn-node wallet export --name <NAME> --show-mnemonic"),
        );
        return Ok(());
    }

    println!();
    println!(
        "  {}",
        style_warn().apply_to("WARNING: This will display sensitive secret material.")
    );
    if !confirm("Continue?")? {
        println!("  Cancelled.");
        return Ok(());
    }

    let password = prompt_password("Enter password")?;

    if show_mnemonic {
        match ks.decrypt_mnemonic(&password)? {
            Some(phrase) => {
                println!();
                println!("  {}", style_bold().apply_to("Mnemonic:"));
                println!("  {}", phrase);
                println!();
            }
            None => {
                print_error(
                    "this wallet was imported from a private key (no mnemonic available)",
                    None,
                );
            }
        }
    }

    if show_private_key {
        let keypair = ks.decrypt_keypair(&password)?;
        let sk_bytes = keypair.signing_key().to_bytes();
        println!();
        println!("  {}", style_bold().apply_to("Private key:"));
        println!("  {}", hex::encode(sk_bytes));
        println!();
    }

    Ok(())
}
