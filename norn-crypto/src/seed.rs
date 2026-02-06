use bip39::Mnemonic;
use norn_types::error::NornError;

/// Generate a new 24-word BIP-39 mnemonic.
pub fn generate_mnemonic() -> Mnemonic {
    // 24 words = 256 bits of entropy = 32 bytes
    let mut entropy = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut entropy);
    Mnemonic::from_entropy(&entropy).expect("32 bytes is valid entropy for 24 words")
}

/// Parse a mnemonic from a string of space-separated words.
pub fn parse_mnemonic(phrase: &str) -> Result<Mnemonic, NornError> {
    Mnemonic::parse_normalized(phrase).map_err(|_| NornError::InvalidMnemonic)
}

/// Derive a 64-byte seed from a mnemonic with an optional passphrase.
/// Uses BIP-39 PBKDF2 derivation.
pub fn mnemonic_to_seed(mnemonic: &Mnemonic, passphrase: &str) -> [u8; 64] {
    mnemonic.to_seed(passphrase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic_is_24_words() {
        let mnemonic = generate_mnemonic();
        let word_count = mnemonic.word_count();
        assert_eq!(word_count, 24);
    }

    #[test]
    fn test_mnemonic_roundtrip() {
        let mnemonic = generate_mnemonic();
        let phrase = mnemonic.to_string();
        let recovered = parse_mnemonic(&phrase).unwrap();
        assert_eq!(mnemonic.to_string(), recovered.to_string());
    }

    #[test]
    fn test_mnemonic_to_seed_deterministic() {
        let mnemonic = generate_mnemonic();
        let seed1 = mnemonic_to_seed(&mnemonic, "");
        let seed2 = mnemonic_to_seed(&mnemonic, "");
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_mnemonic_different_passphrase_different_seed() {
        let mnemonic = generate_mnemonic();
        let seed1 = mnemonic_to_seed(&mnemonic, "");
        let seed2 = mnemonic_to_seed(&mnemonic, "password");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_invalid_mnemonic_rejected() {
        let result = parse_mnemonic("not a valid mnemonic phrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_seed_to_keypair_roundtrip() {
        use crate::keys::Keypair;

        let mnemonic = generate_mnemonic();
        let seed = mnemonic_to_seed(&mnemonic, "");
        let kp1 = Keypair::from_seed(&seed[..32].try_into().unwrap());

        // Recover from same mnemonic
        let phrase = mnemonic.to_string();
        let recovered = parse_mnemonic(&phrase).unwrap();
        let seed2 = mnemonic_to_seed(&recovered, "");
        let kp2 = Keypair::from_seed(&seed2[..32].try_into().unwrap());

        assert_eq!(kp1.public_key(), kp2.public_key());
    }
}
