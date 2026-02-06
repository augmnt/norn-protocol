use libp2p::PeerId;
use norn_types::primitives::Address;
use std::collections::HashMap;

/// Information about a connected peer.
pub struct PeerInfo {
    /// The libp2p peer ID.
    pub peer_id: PeerId,
    /// Optional Norn address (set after registration).
    pub address: Option<Address>,
    /// When this peer connected.
    pub connected_at: std::time::Instant,
}

/// Tracks connected peers and their Norn addresses.
pub struct PeerManager {
    peers: HashMap<PeerId, PeerInfo>,
    address_to_peer: HashMap<Address, PeerId>,
    max_connections: usize,
}

impl PeerManager {
    /// Create a new PeerManager with a maximum connection limit.
    pub fn new(max_connections: usize) -> Self {
        Self {
            peers: HashMap::new(),
            address_to_peer: HashMap::new(),
            max_connections,
        }
    }

    /// Add a peer. Returns false if the connection limit is reached.
    pub fn add_peer(&mut self, peer_id: PeerId) -> bool {
        if self.peers.len() >= self.max_connections {
            return false;
        }
        self.peers.entry(peer_id).or_insert_with(|| PeerInfo {
            peer_id,
            address: None,
            connected_at: std::time::Instant::now(),
        });
        true
    }

    /// Remove a peer and its address mapping.
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        if let Some(info) = self.peers.remove(peer_id) {
            if let Some(addr) = info.address {
                self.address_to_peer.remove(&addr);
            }
        }
    }

    /// Associate a Norn address with a connected peer.
    pub fn register_address(&mut self, peer_id: &PeerId, address: Address) -> bool {
        if let Some(info) = self.peers.get_mut(peer_id) {
            // Remove old mapping if there was a previous address.
            if let Some(old_addr) = info.address.take() {
                self.address_to_peer.remove(&old_addr);
            }
            info.address = Some(address);
            self.address_to_peer.insert(address, *peer_id);
            true
        } else {
            false
        }
    }

    /// Look up a PeerId by Norn address.
    pub fn peer_for_address(&self, address: &Address) -> Option<&PeerId> {
        self.address_to_peer.get(address)
    }

    /// Whether the peer manager has reached its connection limit.
    pub fn is_full(&self) -> bool {
        self.peers.len() >= self.max_connections
    }

    /// Number of currently connected peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Iterator over the peer IDs of all connected peers.
    pub fn connected_peers(&self) -> impl Iterator<Item = &PeerId> {
        self.peers.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer_id() -> PeerId {
        PeerId::random()
    }

    #[test]
    fn test_add_remove_peer() {
        let mut pm = PeerManager::new(10);
        let peer = make_peer_id();
        assert!(pm.add_peer(peer));
        assert_eq!(pm.peer_count(), 1);
        pm.remove_peer(&peer);
        assert_eq!(pm.peer_count(), 0);
    }

    #[test]
    fn test_connection_limit() {
        let mut pm = PeerManager::new(2);
        let p1 = make_peer_id();
        let p2 = make_peer_id();
        let p3 = make_peer_id();
        assert!(pm.add_peer(p1));
        assert!(pm.add_peer(p2));
        assert!(!pm.add_peer(p3));
        assert!(pm.is_full());
        assert_eq!(pm.peer_count(), 2);
    }

    #[test]
    fn test_register_address() {
        let mut pm = PeerManager::new(10);
        let peer = make_peer_id();
        let addr: Address = [42u8; 20];
        pm.add_peer(peer);
        assert!(pm.register_address(&peer, addr));
        assert_eq!(pm.peer_for_address(&addr), Some(&peer));
    }

    #[test]
    fn test_register_address_unknown_peer() {
        let mut pm = PeerManager::new(10);
        let peer = make_peer_id();
        let addr: Address = [42u8; 20];
        assert!(!pm.register_address(&peer, addr));
    }

    #[test]
    fn test_remove_peer_clears_address() {
        let mut pm = PeerManager::new(10);
        let peer = make_peer_id();
        let addr: Address = [42u8; 20];
        pm.add_peer(peer);
        pm.register_address(&peer, addr);
        pm.remove_peer(&peer);
        assert_eq!(pm.peer_for_address(&addr), None);
    }

    #[test]
    fn test_connected_peers() {
        let mut pm = PeerManager::new(10);
        let p1 = make_peer_id();
        let p2 = make_peer_id();
        pm.add_peer(p1);
        pm.add_peer(p2);
        let peers: Vec<_> = pm.connected_peers().cloned().collect();
        assert_eq!(peers.len(), 2);
        assert!(peers.contains(&p1));
        assert!(peers.contains(&p2));
    }

    #[test]
    fn test_re_register_address() {
        let mut pm = PeerManager::new(10);
        let peer = make_peer_id();
        let addr1: Address = [1u8; 20];
        let addr2: Address = [2u8; 20];
        pm.add_peer(peer);
        pm.register_address(&peer, addr1);
        pm.register_address(&peer, addr2);
        // Old address mapping should be removed.
        assert_eq!(pm.peer_for_address(&addr1), None);
        assert_eq!(pm.peer_for_address(&addr2), Some(&peer));
    }
}
