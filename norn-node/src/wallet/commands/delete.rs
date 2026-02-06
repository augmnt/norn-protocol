use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::{print_success, style_warn};
use crate::wallet::keystore::Keystore;
use crate::wallet::prompt::confirm;

pub fn run(name: &str, force: bool) -> Result<(), WalletError> {
    let mut config = WalletConfig::load()?;
    if !config.wallets.contains(&name.to_string()) {
        return Err(WalletError::WalletNotFound(name.to_string()));
    }

    if !force {
        println!(
            "  {}",
            style_warn().apply_to(format!("This will permanently delete wallet '{}'.", name))
        );
        if !confirm("Delete this wallet?")? {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    Keystore::delete(name)?;
    config.remove_wallet(name);
    config.save()?;

    print_success(&format!("Wallet '{}' deleted.", name));
    Ok(())
}
