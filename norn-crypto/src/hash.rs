use norn_types::primitives::Hash;

/// Compute the BLAKE3 hash of the given data.
pub fn blake3_hash(data: &[u8]) -> Hash {
    *blake3::hash(data).as_bytes()
}

/// Compute a BLAKE3 hash with domain separation.
/// The context string ensures different uses of hashing produce different outputs.
pub fn blake3_hash_domain(context: &str, data: &[u8]) -> Hash {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

/// Derive key material using BLAKE3 KDF.
pub fn blake3_kdf(context: &str, key_material: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(key_material);
    *hasher.finalize().as_bytes()
}

/// Hash multiple pieces of data together.
pub fn blake3_hash_multi(parts: &[&[u8]]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hash_deterministic() {
        let data = b"hello norn";
        let h1 = blake3_hash(data);
        let h2 = blake3_hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_blake3_hash_different_inputs() {
        let h1 = blake3_hash(b"hello");
        let h2 = blake3_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_blake3_domain_separation() {
        let data = b"same data";
        let h1 = blake3_hash_domain("context-a", data);
        let h2 = blake3_hash_domain("context-b", data);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_blake3_kdf() {
        let key = blake3_kdf("norn-test", b"key material");
        assert_eq!(key.len(), 32);
        // Deterministic
        let key2 = blake3_kdf("norn-test", b"key material");
        assert_eq!(key, key2);
    }

    #[test]
    fn test_blake3_hash_multi() {
        let h = blake3_hash_multi(&[b"hello", b" ", b"world"]);
        // Should be the same as hashing the concatenation
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"hello");
        hasher.update(b" ");
        hasher.update(b"world");
        let expected = *hasher.finalize().as_bytes();
        assert_eq!(h, expected);
    }
}
