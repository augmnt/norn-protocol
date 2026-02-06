use serde::{Deserialize, Serialize};

use norn_crypto::address::pubkey_to_address;
use norn_crypto::encryption::{decrypt, encrypt_for_keypair, EncryptedMessage};
use norn_crypto::hash::blake3_kdf;
use norn_crypto::hd::derive_default_keypair;
use norn_crypto::keys::Keypair;
use norn_types::primitives::Address;

use super::config::WalletConfig;
use super::error::WalletError;

/// Decryption components extracted from an EncryptedBlob.
type DecryptParts = ([u8; 32], [u8; 24], Vec<u8>);

/// Serializable encrypted blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    pub ephemeral_pubkey: String,
    pub nonce: String,
    pub ciphertext: String,
}

impl EncryptedBlob {
    fn from_encrypted(msg: &EncryptedMessage) -> Self {
        Self {
            ephemeral_pubkey: hex::encode(msg.ephemeral_pubkey),
            nonce: hex::encode(msg.nonce),
            ciphertext: hex::encode(&msg.ciphertext),
        }
    }

    fn to_parts(&self) -> Result<DecryptParts, WalletError> {
        let ephemeral = hex::decode(&self.ephemeral_pubkey)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        let nonce =
            hex::decode(&self.nonce).map_err(|e| WalletError::SerializationError(e.to_string()))?;
        let ciphertext = hex::decode(&self.ciphertext)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;

        let mut eph = [0u8; 32];
        if ephemeral.len() != 32 {
            return Err(WalletError::SerializationError(
                "invalid ephemeral pubkey length".to_string(),
            ));
        }
        eph.copy_from_slice(&ephemeral);

        let mut n = [0u8; 24];
        if nonce.len() != 24 {
            return Err(WalletError::SerializationError(
                "invalid nonce length".to_string(),
            ));
        }
        n.copy_from_slice(&nonce);

        Ok((eph, n, ciphertext))
    }
}

/// On-disk wallet file format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletFile {
    pub version: u32,
    pub name: String,
    pub created_at: u64,
    pub address: String,
    pub public_key: String,
    pub derivation_index: u32,
    pub has_mnemonic: bool,
    pub encrypted_seed: EncryptedBlob,
    pub encrypted_mnemonic: Option<EncryptedBlob>,
}

/// In-memory representation of a loaded wallet.
pub struct Keystore {
    pub name: String,
    pub address: Address,
    pub public_key: [u8; 32],
    pub file: WalletFile,
}

impl Keystore {
    /// Create a new wallet from a mnemonic and password.
    pub fn create(
        name: &str,
        mnemonic: &bip39::Mnemonic,
        passphrase: &str,
        password: &str,
    ) -> Result<Self, WalletError> {
        let seed = norn_crypto::seed::mnemonic_to_seed(mnemonic, passphrase);
        let keypair = derive_default_keypair(&seed)?;
        let address = pubkey_to_address(&keypair.public_key());
        let public_key = keypair.public_key();

        // Derive a password-based keypair for encryption
        let password_keypair = password_to_keypair(password);

        // Encrypt the 64-byte seed
        let encrypted_seed = encrypt_for_keypair(&password_keypair, &seed)?;

        // Encrypt the mnemonic phrase
        let mnemonic_bytes = mnemonic.to_string().into_bytes();
        let encrypted_mnemonic = encrypt_for_keypair(&password_keypair, &mnemonic_bytes)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file = WalletFile {
            version: WALLET_VERSION_ARGON2,
            name: name.to_string(),
            created_at: now,
            address: format!("0x{}", hex::encode(address)),
            public_key: hex::encode(public_key),
            derivation_index: 0,
            has_mnemonic: true,
            encrypted_seed: EncryptedBlob::from_encrypted(&encrypted_seed),
            encrypted_mnemonic: Some(EncryptedBlob::from_encrypted(&encrypted_mnemonic)),
        };

        Ok(Self {
            name: name.to_string(),
            address,
            public_key,
            file,
        })
    }

    /// Create a wallet from a private key seed (32 bytes).
    pub fn from_private_key(
        name: &str,
        seed_bytes: &[u8; 32],
        password: &str,
    ) -> Result<Self, WalletError> {
        let keypair = Keypair::from_seed(seed_bytes);
        let address = pubkey_to_address(&keypair.public_key());
        let public_key = keypair.public_key();

        let password_keypair = password_to_keypair(password);

        // For a private key import, we store the 32-byte seed padded to 64 bytes
        // (only first 32 are meaningful)
        let mut seed_64 = [0u8; 64];
        seed_64[..32].copy_from_slice(seed_bytes);
        let encrypted_seed = encrypt_for_keypair(&password_keypair, &seed_64)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file = WalletFile {
            version: WALLET_VERSION_ARGON2,
            name: name.to_string(),
            created_at: now,
            address: format!("0x{}", hex::encode(address)),
            public_key: hex::encode(public_key),
            derivation_index: 0,
            has_mnemonic: false,
            encrypted_seed: EncryptedBlob::from_encrypted(&encrypted_seed),
            encrypted_mnemonic: None,
        };

        Ok(Self {
            name: name.to_string(),
            address,
            public_key,
            file,
        })
    }

    /// Save the wallet file to disk.
    pub fn save(&self) -> Result<(), WalletError> {
        let dir = WalletConfig::data_dir()?;
        std::fs::create_dir_all(&dir)?;

        // Set directory permissions to 0o700 on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        }

        let path = dir.join(format!("{}.json", self.name));
        let data = serde_json::to_string_pretty(&self.file)?;
        std::fs::write(&path, data)?;

        // Set file permissions to 0o600 on Unix (owner read/write only).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Load a wallet from disk by name.
    pub fn load(name: &str) -> Result<Self, WalletError> {
        let path = WalletConfig::data_dir()?.join(format!("{}.json", name));
        if !path.exists() {
            return Err(WalletError::WalletNotFound(name.to_string()));
        }
        let data = std::fs::read_to_string(&path)?;
        let file: WalletFile = serde_json::from_str(&data)?;

        let address_hex = file.address.strip_prefix("0x").unwrap_or(&file.address);
        let address_bytes =
            hex::decode(address_hex).map_err(|e| WalletError::SerializationError(e.to_string()))?;
        let mut address = [0u8; 20];
        address.copy_from_slice(&address_bytes);

        let pk_bytes = hex::decode(&file.public_key)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(&pk_bytes);

        Ok(Self {
            name: name.to_string(),
            address,
            public_key,
            file,
        })
    }

    /// Delete a wallet file from disk.
    pub fn delete(name: &str) -> Result<(), WalletError> {
        let path = WalletConfig::data_dir()?.join(format!("{}.json", name));
        if !path.exists() {
            return Err(WalletError::WalletNotFound(name.to_string()));
        }
        std::fs::remove_file(path)?;
        Ok(())
    }

    /// List all wallet names on disk.
    pub fn list_names() -> Result<Vec<String>, WalletError> {
        let dir = WalletConfig::data_dir()?;
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem != "config" {
                        names.push(stem.to_string());
                    }
                }
            }
        }
        names.sort();
        Ok(names)
    }

    /// Decrypt the keypair using a password.
    /// Automatically selects the correct KDF based on wallet version.
    /// For v1 wallets, tries Argon2 first then falls back to BLAKE3.
    pub fn decrypt_keypair(&self, password: &str) -> Result<Keypair, WalletError> {
        let password_keypair = password_to_keypair_for_version(password, self.file.version);
        let (eph, nonce, ct) = self.file.encrypted_seed.to_parts()?;
        let seed_bytes = decrypt(&password_keypair, &eph, &nonce, &ct)
            .map_err(|_| WalletError::InvalidPassword)?;

        // Derive the keypair from the seed
        if self.file.has_mnemonic && seed_bytes.len() == 64 {
            let mut seed = [0u8; 64];
            seed.copy_from_slice(&seed_bytes);
            let keypair = derive_default_keypair(&seed)?;
            Ok(keypair)
        } else {
            // Private key import — first 32 bytes are the seed
            let mut seed32 = [0u8; 32];
            seed32.copy_from_slice(&seed_bytes[..32]);
            Ok(Keypair::from_seed(&seed32))
        }
    }

    /// Decrypt the mnemonic phrase if available.
    pub fn decrypt_mnemonic(&self, password: &str) -> Result<Option<String>, WalletError> {
        let enc = match &self.file.encrypted_mnemonic {
            Some(e) => e,
            None => return Ok(None),
        };
        let password_keypair = password_to_keypair_for_version(password, self.file.version);
        let (eph, nonce, ct) = enc.to_parts()?;
        let bytes = decrypt(&password_keypair, &eph, &nonce, &ct)
            .map_err(|_| WalletError::InvalidPassword)?;
        let phrase =
            String::from_utf8(bytes).map_err(|e| WalletError::SerializationError(e.to_string()))?;
        Ok(Some(phrase))
    }
}

/// Wallet version 1 uses BLAKE3 KDF, version 2 uses Argon2id.
const WALLET_VERSION_BLAKE3: u32 = 1;
const WALLET_VERSION_ARGON2: u32 = 2;

/// Derive a keypair from a password using Argon2id (v2 wallets).
fn password_to_keypair_argon2(password: &str) -> Keypair {
    use argon2::Argon2;
    let salt = b"norn-keystore-v2"; // Fixed salt (wallet-specific salt would be better but requires format change)
    let mut seed = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut seed)
        .expect("argon2 hash should succeed");
    Keypair::from_seed(&seed)
}

/// Derive a keypair from a password using BLAKE3 KDF (v1 wallets, legacy).
fn password_to_keypair_blake3(password: &str) -> Keypair {
    let seed = blake3_kdf("norn-keystore-password", password.as_bytes());
    Keypair::from_seed(&seed)
}

/// Derive a keypair from a password, choosing KDF based on wallet version.
fn password_to_keypair(password: &str) -> Keypair {
    // New wallets always use Argon2id (v2).
    password_to_keypair_argon2(password)
}

/// Derive a keypair for decryption, trying the version-appropriate KDF.
fn password_to_keypair_for_version(password: &str, version: u32) -> Keypair {
    if version >= WALLET_VERSION_ARGON2 {
        password_to_keypair_argon2(password)
    } else {
        password_to_keypair_blake3(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::seed::generate_mnemonic;

    #[test]
    fn test_create_and_decrypt_roundtrip() {
        let mnemonic = generate_mnemonic();
        let ks = Keystore::create("test", &mnemonic, "", "mypassword").unwrap();

        let keypair = ks.decrypt_keypair("mypassword").unwrap();
        assert_eq!(keypair.public_key(), ks.public_key);
    }

    #[test]
    fn test_wrong_password_fails() {
        let mnemonic = generate_mnemonic();
        let ks = Keystore::create("test", &mnemonic, "", "correct").unwrap();

        let result = ks.decrypt_keypair("wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_mnemonic_decrypt_roundtrip() {
        let mnemonic = generate_mnemonic();
        let phrase = mnemonic.to_string();
        let ks = Keystore::create("test", &mnemonic, "", "pass").unwrap();

        let recovered = ks.decrypt_mnemonic("pass").unwrap().unwrap();
        assert_eq!(recovered, phrase);
    }

    #[test]
    fn test_from_private_key_roundtrip() {
        let seed = [42u8; 32];
        let ks = Keystore::from_private_key("pk-test", &seed, "pass").unwrap();

        let keypair = ks.decrypt_keypair("pass").unwrap();
        let expected = Keypair::from_seed(&seed);
        assert_eq!(keypair.public_key(), expected.public_key());
    }

    #[test]
    fn test_private_key_no_mnemonic() {
        let seed = [42u8; 32];
        let ks = Keystore::from_private_key("pk-test", &seed, "pass").unwrap();

        let mnemonic = ks.decrypt_mnemonic("pass").unwrap();
        assert!(mnemonic.is_none());
    }

    #[test]
    fn test_encrypted_blob_roundtrip() {
        let password_kp = password_to_keypair("test");
        let plaintext = b"hello world";
        let encrypted = encrypt_for_keypair(&password_kp, plaintext).unwrap();
        let blob = EncryptedBlob::from_encrypted(&encrypted);

        // Serialize and deserialize
        let json = serde_json::to_string(&blob).unwrap();
        let recovered: EncryptedBlob = serde_json::from_str(&json).unwrap();

        let (eph, nonce, ct) = recovered.to_parts().unwrap();
        let decrypted = decrypt(&password_kp, &eph, &nonce, &ct).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
