use norn_types::network::SpindleRegistration;
use norn_types::primitives::Address;
use std::collections::HashMap;

/// Registry of spindles known to this relay node.
pub struct SpindleRegistry {
    spindles: HashMap<Address, SpindleRegistration>,
}

impl SpindleRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            spindles: HashMap::new(),
        }
    }

    /// Register a spindle. Overwrites any previous registration for the same address.
    pub fn register(&mut self, reg: SpindleRegistration) {
        self.spindles.insert(reg.address, reg);
    }

    /// Unregister a spindle by address.
    pub fn unregister(&mut self, address: &Address) -> bool {
        self.spindles.remove(address).is_some()
    }

    /// Get a spindle registration by address.
    pub fn get(&self, address: &Address) -> Option<&SpindleRegistration> {
        self.spindles.get(address)
    }

    /// List all registered spindles.
    pub fn list(&self) -> Vec<&SpindleRegistration> {
        self.spindles.values().collect()
    }

    /// Check whether a spindle is registered.
    pub fn is_registered(&self, address: &Address) -> bool {
        self.spindles.contains_key(address)
    }
}

impl Default for SpindleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::network::SpindleRegistration;

    fn make_registration(addr_byte: u8) -> SpindleRegistration {
        SpindleRegistration {
            pubkey: [addr_byte; 32],
            address: [addr_byte; 20],
            relay_endpoint: format!("127.0.0.1:{}", 9740 + addr_byte as u16),
            timestamp: 1000,
            signature: [addr_byte; 64],
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = SpindleRegistry::new();
        let reg = make_registration(1);
        let addr = reg.address;
        registry.register(reg);
        assert!(registry.is_registered(&addr));
        assert!(registry.get(&addr).is_some());
    }

    #[test]
    fn test_unregister() {
        let mut registry = SpindleRegistry::new();
        let reg = make_registration(1);
        let addr = reg.address;
        registry.register(reg);
        assert!(registry.unregister(&addr));
        assert!(!registry.is_registered(&addr));
        // Double unregister returns false.
        assert!(!registry.unregister(&addr));
    }

    #[test]
    fn test_list() {
        let mut registry = SpindleRegistry::new();
        registry.register(make_registration(1));
        registry.register(make_registration(2));
        registry.register(make_registration(3));
        assert_eq!(registry.list().len(), 3);
    }

    #[test]
    fn test_overwrite_registration() {
        let mut registry = SpindleRegistry::new();
        let mut reg1 = make_registration(1);
        reg1.relay_endpoint = "old".to_string();
        let addr = reg1.address;
        registry.register(reg1);

        let mut reg2 = make_registration(1);
        reg2.relay_endpoint = "new".to_string();
        registry.register(reg2);

        assert_eq!(registry.get(&addr).unwrap().relay_endpoint, "new");
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let registry = SpindleRegistry::new();
        let addr = [99u8; 20];
        assert!(registry.get(&addr).is_none());
        assert!(!registry.is_registered(&addr));
    }
}
