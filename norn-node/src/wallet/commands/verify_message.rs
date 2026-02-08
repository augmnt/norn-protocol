use crate::wallet::error::WalletError;
use crate::wallet::format::{print_error, print_success};

pub fn run(message: &str, signature_hex: &str, pubkey_hex: &str) -> Result<(), WalletError> {
    // Parse hex signature (64 bytes)
    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| WalletError::Other(format!("invalid signature hex: {}", e)))?;
    if sig_bytes.len() != 64 {
        return Err(WalletError::Other(format!(
            "signature must be 64 bytes, got {}",
            sig_bytes.len()
        )));
    }
    let mut signature = [0u8; 64];
    signature.copy_from_slice(&sig_bytes);

    // Parse hex pubkey (32 bytes)
    let pk_bytes = hex::decode(pubkey_hex)
        .map_err(|e| WalletError::Other(format!("invalid pubkey hex: {}", e)))?;
    if pk_bytes.len() != 32 {
        return Err(WalletError::Other(format!(
            "public key must be 32 bytes, got {}",
            pk_bytes.len()
        )));
    }
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&pk_bytes);

    // Hash the message with BLAKE3 (same as sign-message)
    let hash = norn_crypto::hash::blake3_hash(message.as_bytes());

    // Verify the signature
    match norn_crypto::keys::verify(&hash, &signature, &pubkey) {
        Ok(()) => {
            println!();
            print_success("Signature valid");
            println!();
        }
        Err(_) => {
            println!();
            print_error("Signature INVALID", None);
            println!();
        }
    }

    Ok(())
}
