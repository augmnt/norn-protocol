use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::print_success;
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::{prompt_new_password, prompt_password};

pub fn run(name: Option<&str>) -> Result<(), WalletError> {
    let config = WalletConfig::load()?;
    let wallet_name = match name {
        Some(n) => n,
        None => config.active_wallet_name()?,
    };

    let mut ks = Keystore::load(wallet_name)?;

    let old_password = prompt_password("Enter current password")?;

    // Verify old password works before prompting for new one
    ks.decrypt_keypair(&old_password)?;

    let new_password = prompt_new_password()?;

    ks.change_password(&old_password, &new_password)?;

    println!();
    print_success(&format!("Password changed for wallet '{}'", wallet_name));
    println!();

    Ok(())
}
