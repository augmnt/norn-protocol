use norn_types::primitives::{Address, PublicKey};

use crate::hash::blake3_hash;

/// Derive an address from a public key.
/// Address = BLAKE3(pubkey)[0..20]
pub fn pubkey_to_address(pubkey: &PublicKey) -> Address {
    let hash = blake3_hash(pubkey);
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[..20]);
    address
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_derivation_deterministic() {
        let pubkey = [42u8; 32];
        let addr1 = pubkey_to_address(&pubkey);
        let addr2 = pubkey_to_address(&pubkey);
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_different_pubkeys_different_addresses() {
        let pk1 = [1u8; 32];
        let pk2 = [2u8; 32];
        let addr1 = pubkey_to_address(&pk1);
        let addr2 = pubkey_to_address(&pk2);
        assert_ne!(addr1, addr2);
    }

    #[test]
    fn test_address_length() {
        let pubkey = [99u8; 32];
        let addr = pubkey_to_address(&pubkey);
        assert_eq!(addr.len(), 20);
    }
}
