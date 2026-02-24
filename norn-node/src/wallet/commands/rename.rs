use crate::wallet::config::WalletConfig;
use crate::wallet::error::WalletError;
use crate::wallet::format::print_success;
use crate::wallet::keystore::validate_wallet_name;

pub fn run(from: &str, to: &str) -> Result<(), WalletError> {
    validate_wallet_name(from)?;
    validate_wallet_name(to)?;
    let wallet_dir = WalletConfig::data_dir()?;
    let source = wallet_dir.join(format!("{}.json", from));
    let target = wallet_dir.join(format!("{}.json", to));

    if !source.exists() {
        return Err(WalletError::WalletNotFound(from.to_string()));
    }
    if target.exists() {
        return Err(WalletError::WalletAlreadyExists(to.to_string()));
    }

    std::fs::rename(&source, &target)?;

    // Update the wallet file's internal name field.
    let data = std::fs::read_to_string(&target)?;
    let mut file: crate::wallet::keystore::WalletFile = serde_json::from_str(&data)?;
    file.name = to.to_string();
    let updated = serde_json::to_string_pretty(&file)?;
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&target)?;
        f.write_all(updated.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&target, updated)?;
    }

    // Update config: rename in wallets list and update active wallet if needed.
    let mut config = WalletConfig::load()?;
    let was_active = config.active_wallet.as_deref() == Some(from);
    config.wallets.retain(|n| n != from);
    if !config.wallets.contains(&to.to_string()) {
        config.wallets.push(to.to_string());
    }
    if was_active {
        config.active_wallet = Some(to.to_string());
    }
    config.save()?;

    println!();
    print_success(&format!("Renamed wallet '{}' to '{}'", from, to));
    if was_active {
        println!("  Active wallet updated to '{}'", to);
    }
    println!();

    Ok(())
}
