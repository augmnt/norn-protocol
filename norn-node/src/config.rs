use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::NodeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Network identity: "dev", "testnet", or "mainnet".
    #[serde(default = "default_network_id")]
    pub network_id: String,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub validator: ValidatorConfig,
    pub rpc: RpcConfig,
    pub logging: LoggingConfig,
    /// Path to a genesis file. If set, load genesis state from this file.
    #[serde(default)]
    pub genesis_path: Option<String>,
    /// Inline genesis config (programmatic only, not serialized to TOML).
    #[serde(skip)]
    pub genesis_config: Option<norn_types::genesis::GenesisConfig>,
}

fn default_network_id() -> String {
    "dev".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub boot_nodes: Vec<String>,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
    /// Storage backend: "memory", "sqlite", or "rocksdb"
    pub db_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub enabled: bool,
    pub keypair_path: Option<String>,
    /// Hex-encoded 32-byte seed for deterministic keypair generation.
    pub keypair_seed: Option<String>,
    /// When true, produce blocks directly without HotStuff consensus (solo/dev mode).
    #[serde(default = "default_solo_mode")]
    pub solo_mode: bool,
}

fn default_solo_mode() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub enabled: bool,
    pub listen_addr: String,
    pub max_connections: usize,
    /// Optional API key for RPC authentication.
    /// If set, mutation methods require `Authorization: Bearer <key>` header.
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network_id: default_network_id(),
            network: NetworkConfig {
                listen_addr: "0.0.0.0:9740".to_string(),
                boot_nodes: norn_types::constants::DEFAULT_BOOT_NODES
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                max_connections: 50,
            },
            storage: StorageConfig {
                data_dir: dirs::home_dir()
                    .map(|h| h.join(".norn").join("data").to_string_lossy().into_owned())
                    .unwrap_or_else(|| "./norn-data".to_string()),
                db_type: "memory".to_string(),
            },
            validator: ValidatorConfig {
                enabled: false,
                keypair_path: None,
                keypair_seed: None,
                solo_mode: false,
            },
            rpc: RpcConfig {
                enabled: true,
                listen_addr: "127.0.0.1:9741".to_string(),
                max_connections: 100,
                api_key: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
            genesis_path: None,
            genesis_config: None,
        }
    }
}

impl NodeConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: &str) -> Result<Self, NodeError> {
        let contents = std::fs::read_to_string(path).map_err(|e| NodeError::ConfigError {
            reason: format!("failed to read config file '{}': {}", path, e),
        })?;
        let config: NodeConfig = toml::from_str(&contents).map_err(|e| NodeError::ConfigError {
            reason: format!("failed to parse config file '{}': {}", path, e),
        })?;
        Ok(config)
    }

    /// Initialize a default configuration file in the given directory.
    pub fn init(dir: &str) -> Result<(), NodeError> {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)?;
        }

        let config = NodeConfig::default();
        let toml_str = toml::to_string_pretty(&config).map_err(|e| NodeError::ConfigError {
            reason: format!("failed to serialize default config: {}", e),
        })?;

        let config_path = dir_path.join("norn.toml");
        std::fs::write(&config_path, toml_str)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert_eq!(config.network.listen_addr, "0.0.0.0:9740");
        assert_eq!(config.storage.db_type, "memory");
        assert!(!config.validator.enabled);
        assert!(config.rpc.enabled);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = NodeConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: NodeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.network.listen_addr, config.network.listen_addr);
        assert_eq!(deserialized.storage.db_type, config.storage.db_type);
        assert_eq!(deserialized.rpc.listen_addr, config.rpc.listen_addr);
    }

    #[test]
    fn test_init_creates_config_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_str().unwrap();
        NodeConfig::init(dir).unwrap();

        let config_path = tmp.path().join("norn.toml");
        assert!(config_path.exists());

        let contents = std::fs::read_to_string(config_path).unwrap();
        let _config: NodeConfig = toml::from_str(&contents).unwrap();
    }

    #[test]
    fn test_default_config_has_boot_nodes() {
        let config = NodeConfig::default();
        assert!(!config.network.boot_nodes.is_empty());
        assert!(config.network.boot_nodes[0].contains("seed.norn.network"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = NodeConfig::load("/nonexistent/path/norn.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_valid_config() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_str().unwrap();
        NodeConfig::init(dir).unwrap();

        let config_path = tmp.path().join("norn.toml");
        let config = NodeConfig::load(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.network.listen_addr, "0.0.0.0:9740");
    }
}
