use libp2p::Multiaddr;

/// Handles bootstrap peer discovery.
pub struct Discovery {
    boot_nodes: Vec<Multiaddr>,
}

impl Discovery {
    /// Create a new Discovery from a list of multiaddr strings.
    /// Invalid multiaddr strings are logged and skipped.
    pub fn new(boot_nodes: Vec<String>) -> Self {
        let addrs = boot_nodes
            .iter()
            .filter_map(|s| {
                s.parse::<Multiaddr>()
                    .map_err(|e| {
                        tracing::warn!("Invalid multiaddr '{}': {}", s, e);
                        e
                    })
                    .ok()
            })
            .collect();

        Self { boot_nodes: addrs }
    }

    /// Return the parsed bootstrap addresses.
    pub fn boot_addrs(&self) -> &[Multiaddr] {
        &self.boot_nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_multiaddr() {
        let addrs = vec!["/ip4/127.0.0.1/tcp/9740".to_string()];
        let disc = Discovery::new(addrs);
        assert_eq!(disc.boot_addrs().len(), 1);
    }

    #[test]
    fn test_parse_invalid_multiaddr() {
        let addrs = vec!["not-a-multiaddr".to_string()];
        let disc = Discovery::new(addrs);
        assert_eq!(disc.boot_addrs().len(), 0);
    }

    #[test]
    fn test_empty() {
        let disc = Discovery::new(vec![]);
        assert!(disc.boot_addrs().is_empty());
    }
}
