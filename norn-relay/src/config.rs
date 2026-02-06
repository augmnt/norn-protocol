use std::net::SocketAddr;

use norn_types::constants::{DEFAULT_RELAY_PORT, MAX_RELAY_CONNECTIONS};

/// Configuration for a relay node.
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Address to listen on.
    pub listen_addr: SocketAddr,
    /// Bootstrap node addresses (multiaddr strings).
    pub boot_nodes: Vec<String>,
    /// Maximum number of connections.
    pub max_connections: usize,
    /// Optional keypair seed (32 bytes). If None, generates random.
    pub keypair_seed: Option<[u8; 32]>,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            listen_addr: ([0, 0, 0, 0], DEFAULT_RELAY_PORT).into(),
            boot_nodes: Vec::new(),
            max_connections: MAX_RELAY_CONNECTIONS,
            keypair_seed: None,
        }
    }
}
