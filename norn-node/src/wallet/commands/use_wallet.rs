use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::print_success;

pub fn run(name: &str) -> Result<(), WalletError> {
    let mut config = WalletConfig::load()?;
    config.set_active(name)?;
    config.save()?;
    print_success(&format!("Active wallet set to '{}'", name));
    Ok(())
}
