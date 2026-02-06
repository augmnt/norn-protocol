use ed25519_dalek::{Signer, Verifier};
use norn_types::error::NornError;
use norn_types::primitives::{PublicKey, Signature};

/// Wrapper around an Ed25519 keypair.
pub struct Keypair {
    inner: ed25519_dalek::SigningKey,
}

impl Keypair {
    /// Generate a new random keypair.
    pub fn generate() -> Self {
        let mut csprng = rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        Self { inner: signing_key }
    }

    /// Create a keypair from a 32-byte seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(seed);
        Self { inner: signing_key }
    }

    /// Get the public key bytes.
    pub fn public_key(&self) -> PublicKey {
        self.inner.verifying_key().to_bytes()
    }

    /// Get a reference to the underlying signing key.
    pub fn signing_key(&self) -> &ed25519_dalek::SigningKey {
        &self.inner
    }

    /// Get the 32-byte seed (secret key bytes) of this keypair.
    pub fn seed(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }

    /// Sign a message, returning the 64-byte signature.
    pub fn sign(&self, message: &[u8]) -> Signature {
        let sig = self.inner.sign(message);
        sig.to_bytes()
    }
}

// Note: SigningKey with the "zeroize" feature implements ZeroizeOnDrop,
// so key material is automatically wiped when Keypair is dropped.

/// Verify an Ed25519 signature.
pub fn verify(message: &[u8], signature: &Signature, pubkey: &PublicKey) -> Result<(), NornError> {
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(pubkey)
        .map_err(|_| NornError::InvalidKeyMaterial)?;
    let sig = ed25519_dalek::Signature::from_bytes(signature);
    verifying_key
        .verify(message, &sig)
        .map_err(|_| NornError::InvalidSignature { signer_index: 0 })
}

/// Batch-verify multiple signatures using ed25519-dalek's true batch verification.
///
/// Uses a fast probabilistic batch verification first. If the batch fails,
/// falls back to sequential verification to identify the specific failing index.
/// Returns Ok(()) if all signatures are valid, or the first error encountered.
pub fn batch_verify(
    messages: &[&[u8]],
    signatures: &[Signature],
    pubkeys: &[PublicKey],
) -> Result<(), NornError> {
    if messages.len() != signatures.len() || messages.len() != pubkeys.len() {
        return Err(NornError::InvalidSignature { signer_index: 0 });
    }
    if messages.is_empty() {
        return Ok(());
    }

    // Parse all keys and signatures up front.
    let mut verifying_keys = Vec::with_capacity(pubkeys.len());
    let mut dalek_sigs = Vec::with_capacity(signatures.len());
    for (i, (pk, sig)) in pubkeys.iter().zip(signatures.iter()).enumerate() {
        let vk = ed25519_dalek::VerifyingKey::from_bytes(pk)
            .map_err(|_| NornError::InvalidSignature { signer_index: i })?;
        verifying_keys.push(vk);
        dalek_sigs.push(ed25519_dalek::Signature::from_bytes(sig));
    }

    // Fast path: true batch verification (probabilistic, much faster for large batches).
    if ed25519_dalek::verify_batch(messages, &dalek_sigs, &verifying_keys).is_ok() {
        return Ok(());
    }

    // Slow path: the batch failed, find the specific invalid signature.
    for (i, ((msg, sig), vk)) in messages
        .iter()
        .zip(dalek_sigs.iter())
        .zip(verifying_keys.iter())
        .enumerate()
    {
        if vk.verify(msg, sig).is_err() {
            return Err(NornError::InvalidSignature { signer_index: i });
        }
    }

    // Batch verification can fail even if individual verifications pass (probabilistic).
    // In that extremely rare case, treat it as valid since all individual checks passed.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify_roundtrip() {
        let kp = Keypair::generate();
        let msg = b"hello norn";
        let sig = kp.sign(msg);
        assert!(verify(msg, &sig, &kp.public_key()).is_ok());
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let kp = Keypair::generate();
        let msg = b"hello norn";
        let mut sig = kp.sign(msg);
        sig[0] ^= 0xff; // Corrupt the signature
        assert!(verify(msg, &sig, &kp.public_key()).is_err());
    }

    #[test]
    fn test_wrong_message_rejected() {
        let kp = Keypair::generate();
        let sig = kp.sign(b"hello norn");
        assert!(verify(b"wrong message", &sig, &kp.public_key()).is_err());
    }

    #[test]
    fn test_wrong_pubkey_rejected() {
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();
        let msg = b"hello norn";
        let sig = kp1.sign(msg);
        assert!(verify(msg, &sig, &kp2.public_key()).is_err());
    }

    #[test]
    fn test_from_seed_deterministic() {
        let seed = [42u8; 32];
        let kp1 = Keypair::from_seed(&seed);
        let kp2 = Keypair::from_seed(&seed);
        assert_eq!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn test_batch_verify() {
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();
        let msg1 = b"message one";
        let msg2 = b"message two";
        let sig1 = kp1.sign(msg1);
        let sig2 = kp2.sign(msg2);

        assert!(batch_verify(
            &[msg1.as_slice(), msg2.as_slice()],
            &[sig1, sig2],
            &[kp1.public_key(), kp2.public_key()],
        )
        .is_ok());
    }

    #[test]
    fn test_batch_verify_one_invalid() {
        let kp1 = Keypair::generate();
        let kp2 = Keypair::generate();
        let msg1 = b"message one";
        let msg2 = b"message two";
        let sig1 = kp1.sign(msg1);
        let mut sig2 = kp2.sign(msg2);
        sig2[0] ^= 0xff;

        let result = batch_verify(
            &[msg1.as_slice(), msg2.as_slice()],
            &[sig1, sig2],
            &[kp1.public_key(), kp2.public_key()],
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            NornError::InvalidSignature { signer_index } => assert_eq!(signer_index, 1),
            _ => panic!("Expected InvalidSignature error"),
        }
    }
}
