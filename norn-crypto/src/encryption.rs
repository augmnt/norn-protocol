use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use norn_types::error::NornError;
use rand::RngCore;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

use crate::keys::Keypair;

/// The result of an encryption operation.
pub struct EncryptedMessage {
    /// Ephemeral X25519 public key (32 bytes).
    pub ephemeral_pubkey: [u8; 32],
    /// XChaCha20-Poly1305 nonce (24 bytes).
    pub nonce: [u8; 24],
    /// Encrypted ciphertext with authentication tag.
    pub ciphertext: Vec<u8>,
}

/// Derive an X25519 static secret from an Ed25519 keypair using BLAKE3 KDF.
/// Both encrypt and decrypt use this same derivation so the shared secrets match.
fn keypair_to_x25519_secret(keypair: &Keypair) -> StaticSecret {
    let sk_bytes = keypair.signing_key().to_bytes();
    let x_secret = crate::hash::blake3_kdf("norn-ed25519-to-x25519", &sk_bytes);
    StaticSecret::from(x_secret)
}

/// Derive an X25519 public key from an Ed25519 keypair's X25519 secret.
fn keypair_to_x25519_public(keypair: &Keypair) -> X25519Public {
    let secret = keypair_to_x25519_secret(keypair);
    X25519Public::from(&secret)
}

/// Encrypt a plaintext message for a recipient.
///
/// The recipient_keypair_public is the X25519 public key derived from the
/// recipient's Ed25519 keypair. Since we can't derive the X25519 public from
/// just the Ed25519 public key in a compatible way, callers should use
/// `encrypt_for_keypair` or pass the pre-derived X25519 public key.
///
/// Uses ephemeral X25519 Diffie-Hellman + XChaCha20-Poly1305 AEAD.
pub fn encrypt(
    recipient_x25519_public: &[u8; 32],
    plaintext: &[u8],
) -> Result<EncryptedMessage, NornError> {
    let recipient_x = X25519Public::from(*recipient_x25519_public);

    // Generate ephemeral X25519 keypair
    let mut rng = rand::rngs::OsRng;
    let ephemeral_secret = StaticSecret::random_from_rng(rng);
    let ephemeral_public = X25519Public::from(&ephemeral_secret);

    // Diffie-Hellman shared secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_x);

    // Derive encryption key from shared secret using BLAKE3 KDF
    let encryption_key = crate::hash::blake3_kdf("norn-encryption-key", shared_secret.as_bytes());

    // Generate random nonce
    let mut nonce_bytes = [0u8; 24];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    // Encrypt
    let cipher = XChaCha20Poly1305::new_from_slice(&encryption_key).map_err(|e| {
        NornError::EncryptionFailed {
            reason: e.to_string(),
        }
    })?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| NornError::EncryptionFailed {
            reason: e.to_string(),
        })?;

    Ok(EncryptedMessage {
        ephemeral_pubkey: ephemeral_public.to_bytes(),
        nonce: nonce_bytes,
        ciphertext,
    })
}

/// Encrypt a plaintext message for a recipient identified by their Ed25519 keypair.
/// This is the primary high-level encryption function.
pub fn encrypt_for_keypair(
    recipient: &Keypair,
    plaintext: &[u8],
) -> Result<EncryptedMessage, NornError> {
    let x_public = keypair_to_x25519_public(recipient);
    encrypt(&x_public.to_bytes(), plaintext)
}

/// Get the X25519 public key for a keypair (to share with others for encryption).
pub fn x25519_public_key(keypair: &Keypair) -> [u8; 32] {
    keypair_to_x25519_public(keypair).to_bytes()
}

/// Decrypt a message using the recipient's Ed25519 keypair.
pub fn decrypt(
    keypair: &Keypair,
    ephemeral_pubkey: &[u8; 32],
    nonce: &[u8; 24],
    ciphertext: &[u8],
) -> Result<Vec<u8>, NornError> {
    let recipient_secret = keypair_to_x25519_secret(keypair);
    let ephemeral_x = X25519Public::from(*ephemeral_pubkey);

    // Diffie-Hellman shared secret
    let shared_secret = recipient_secret.diffie_hellman(&ephemeral_x);

    // Derive encryption key
    let encryption_key = crate::hash::blake3_kdf("norn-encryption-key", shared_secret.as_bytes());

    // Decrypt
    let cipher = XChaCha20Poly1305::new_from_slice(&encryption_key).map_err(|e| {
        NornError::DecryptionFailed {
            reason: e.to_string(),
        }
    })?;
    let nonce = XNonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| NornError::DecryptionFailed {
            reason: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let recipient = Keypair::generate();
        let plaintext = b"hello, encrypted norn world!";

        let encrypted = encrypt_for_keypair(&recipient, plaintext).unwrap();
        let decrypted = decrypt(
            &recipient,
            &encrypted.ephemeral_pubkey,
            &encrypted.nonce,
            &encrypted.ciphertext,
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_decryption_fails() {
        let recipient = Keypair::generate();
        let wrong_recipient = Keypair::generate();
        let plaintext = b"secret message";

        let encrypted = encrypt_for_keypair(&recipient, plaintext).unwrap();
        let result = decrypt(
            &wrong_recipient,
            &encrypted.ephemeral_pubkey,
            &encrypted.nonce,
            &encrypted.ciphertext,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let recipient = Keypair::generate();
        let plaintext = b"secret message";

        let mut encrypted = encrypt_for_keypair(&recipient, plaintext).unwrap();
        if let Some(byte) = encrypted.ciphertext.first_mut() {
            *byte ^= 0xff;
        }

        let result = decrypt(
            &recipient,
            &encrypted.ephemeral_pubkey,
            &encrypted.nonce,
            &encrypted.ciphertext,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let recipient = Keypair::generate();
        let plaintext = b"";

        let encrypted = encrypt_for_keypair(&recipient, plaintext).unwrap();
        let decrypted = decrypt(
            &recipient,
            &encrypted.ephemeral_pubkey,
            &encrypted.nonce,
            &encrypted.ciphertext,
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let recipient = Keypair::generate();
        let plaintext = vec![0xABu8; 65536];

        let encrypted = encrypt_for_keypair(&recipient, &plaintext).unwrap();
        let decrypted = decrypt(
            &recipient,
            &encrypted.ephemeral_pubkey,
            &encrypted.nonce,
            &encrypted.ciphertext,
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
