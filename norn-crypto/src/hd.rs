use norn_types::constants::NORN_COIN_TYPE;
use norn_types::error::NornError;

use crate::keys::Keypair;

/// Derive an Ed25519 keypair using SLIP-0010 from a BIP-39 seed.
///
/// Path: m/44'/{NORN_COIN_TYPE}'/0'/0'/{index}'
///
/// All path components are hardened (required for Ed25519 by SLIP-0010).
pub fn derive_keypair(seed: &[u8; 64], index: u32) -> Result<Keypair, NornError> {
    let path = [44, NORN_COIN_TYPE, 0, 0, index];

    let derived = slip10_ed25519::derive_ed25519_private_key(seed, &path);
    Ok(Keypair::from_seed(&derived))
}

/// Derive a keypair at the default index (0).
pub fn derive_default_keypair(seed: &[u8; 64]) -> Result<Keypair, NornError> {
    derive_keypair(seed, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::{generate_mnemonic, mnemonic_to_seed};

    #[test]
    fn test_derive_keypair_deterministic() {
        let mnemonic = generate_mnemonic();
        let seed = mnemonic_to_seed(&mnemonic, "");
        let kp1 = derive_keypair(&seed, 0).unwrap();
        let kp2 = derive_keypair(&seed, 0).unwrap();
        assert_eq!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn test_different_indices_different_keys() {
        let mnemonic = generate_mnemonic();
        let seed = mnemonic_to_seed(&mnemonic, "");
        let kp0 = derive_keypair(&seed, 0).unwrap();
        let kp1 = derive_keypair(&seed, 1).unwrap();
        assert_ne!(kp0.public_key(), kp1.public_key());
    }

    #[test]
    fn test_different_seeds_different_keys() {
        let m1 = generate_mnemonic();
        let m2 = generate_mnemonic();
        let s1 = mnemonic_to_seed(&m1, "");
        let s2 = mnemonic_to_seed(&m2, "");
        let kp1 = derive_keypair(&s1, 0).unwrap();
        let kp2 = derive_keypair(&s2, 0).unwrap();
        assert_ne!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn test_derive_default_keypair() {
        let mnemonic = generate_mnemonic();
        let seed = mnemonic_to_seed(&mnemonic, "");
        let kp_default = derive_default_keypair(&seed).unwrap();
        let kp_zero = derive_keypair(&seed, 0).unwrap();
        assert_eq!(kp_default.public_key(), kp_zero.public_key());
    }
}
