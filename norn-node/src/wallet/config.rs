use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::error::WalletError;

const DEFAULT_RPC_URL: &str = "http://127.0.0.1:9741";

/// Wallet configuration stored in ~/.norn/wallets/config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Name of the currently active wallet.
    pub active_wallet: Option<String>,
    /// RPC endpoint URL.
    pub rpc_url: String,
    /// Network identifier: "dev", "testnet", "mainnet".
    #[serde(default = "default_network")]
    pub network: String,
    /// List of known wallet names.
    pub wallets: Vec<String>,
}

fn default_network() -> String {
    "dev".to_string()
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            active_wallet: None,
            rpc_url: DEFAULT_RPC_URL.to_string(),
            network: default_network(),
            wallets: Vec::new(),
        }
    }
}

impl WalletConfig {
    /// Get the wallet data directory (~/.norn/wallets/).
    pub fn data_dir() -> Result<PathBuf, WalletError> {
        let home = dirs::home_dir().ok_or_else(|| {
            WalletError::ConfigError("could not determine home directory".to_string())
        })?;
        Ok(home.join(".norn").join("wallets"))
    }

    /// Get the config file path.
    fn config_path() -> Result<PathBuf, WalletError> {
        Ok(Self::data_dir()?.join("config.json"))
    }

    /// Load config from disk, or create default if it doesn't exist.
    pub fn load() -> Result<Self, WalletError> {
        let path = Self::config_path()?;
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            let config: WalletConfig = serde_json::from_str(&data)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), WalletError> {
        let dir = Self::data_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path()?;
        let data = serde_json::to_string_pretty(self)?;

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)?;
            file.write_all(data.as_bytes())?;
        }

        #[cfg(not(unix))]
        {
            std::fs::write(path, data)?;
        }

        Ok(())
    }

    /// Get the active wallet name, or error if none set.
    pub fn active_wallet_name(&self) -> Result<&str, WalletError> {
        self.active_wallet
            .as_deref()
            .ok_or(WalletError::NoActiveWallet)
    }

    /// Add a wallet name to the registry.
    pub fn add_wallet(&mut self, name: &str) {
        if !self.wallets.contains(&name.to_string()) {
            self.wallets.push(name.to_string());
        }
    }

    /// Remove a wallet name from the registry.
    pub fn remove_wallet(&mut self, name: &str) {
        self.wallets.retain(|n| n != name);
        if self.active_wallet.as_deref() == Some(name) {
            self.active_wallet = None;
        }
    }

    /// Set the active wallet.
    pub fn set_active(&mut self, name: &str) -> Result<(), WalletError> {
        if !self.wallets.contains(&name.to_string()) {
            return Err(WalletError::WalletNotFound(name.to_string()));
        }
        self.active_wallet = Some(name.to_string());
        Ok(())
    }
}
