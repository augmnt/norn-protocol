//! Shamir's Secret Sharing for social recovery of seed phrases.
//!
//! Splits a secret (typically a 32-byte seed or 64-byte BIP-39 seed) into `n`
//! shares, of which any `k` are sufficient to reconstruct the original secret.

use norn_types::error::NornError;
use sharks::{Share, Sharks};

/// A single share from Shamir's Secret Sharing.
/// Contains the share index and the share data.
#[derive(Debug, Clone)]
pub struct ShamirShare {
    /// The raw share bytes (can be serialized/deserialized).
    pub data: Vec<u8>,
}

/// Split a secret into `n` shares, requiring `k` shares to reconstruct.
///
/// # Arguments
/// * `secret` - The secret bytes to split (e.g., 32-byte seed or 64-byte BIP-39 seed).
/// * `threshold` - Minimum number of shares required to reconstruct (`k`).
/// * `total` - Total number of shares to generate (`n`).
///
/// # Errors
/// Returns an error if `threshold` > `total`, `threshold` < 2, or `total` < 2.
pub fn split_secret(
    secret: &[u8],
    threshold: u8,
    total: u8,
) -> Result<Vec<ShamirShare>, NornError> {
    if threshold < 2 {
        return Err(NornError::ShamirError {
            reason: "threshold must be at least 2".to_string(),
        });
    }
    if total < threshold {
        return Err(NornError::ShamirError {
            reason: "total shares must be >= threshold".to_string(),
        });
    }
    if secret.is_empty() {
        return Err(NornError::ShamirError {
            reason: "secret must not be empty".to_string(),
        });
    }

    let sharks = Sharks(threshold);
    let dealer = sharks.dealer(secret);
    let shares: Vec<ShamirShare> = dealer
        .take(total as usize)
        .map(|share| ShamirShare {
            data: Vec::from(&share),
        })
        .collect();

    Ok(shares)
}

/// Reconstruct a secret from `k` or more shares.
///
/// # Arguments
/// * `shares` - At least `threshold` shares from the original split.
///
/// # Errors
/// Returns an error if there are insufficient shares or the shares are invalid.
pub fn reconstruct_secret(shares: &[ShamirShare]) -> Result<Vec<u8>, NornError> {
    if shares.len() < 2 {
        return Err(NornError::ShamirError {
            reason: "need at least 2 shares to reconstruct".to_string(),
        });
    }

    let shark_shares: Vec<Share> = shares
        .iter()
        .map(|s| Share::try_from(s.data.as_slice()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| NornError::ShamirError {
            reason: format!("invalid share data: {}", e),
        })?;

    // The Sharks struct needs the threshold, but recover() infers it from the shares.
    // We use threshold=2 as a minimum; the actual threshold is encoded in the shares.
    let sharks = Sharks(2);
    let secret = sharks
        .recover(&shark_shares)
        .map_err(|e| NornError::ShamirError {
            reason: format!("failed to reconstruct secret: {}", e),
        })?;

    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_reconstruct_3_of_5() {
        let secret = b"this is a 32-byte secret key!!!";
        let shares = split_secret(secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Reconstruct from first 3 shares.
        let recovered = reconstruct_secret(&shares[..3]).unwrap();
        assert_eq!(recovered, secret);

        // Reconstruct from last 3 shares.
        let recovered = reconstruct_secret(&shares[2..]).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_split_and_reconstruct_2_of_3() {
        let secret = [42u8; 32];
        let shares = split_secret(&secret, 2, 3).unwrap();
        assert_eq!(shares.len(), 3);

        let recovered = reconstruct_secret(&shares[..2]).unwrap();
        assert_eq!(recovered, secret.to_vec());
    }

    #[test]
    fn test_split_and_reconstruct_64_byte_seed() {
        let secret = [0xABu8; 64];
        let shares = split_secret(&secret, 3, 5).unwrap();
        let recovered = reconstruct_secret(&shares[1..4]).unwrap();
        assert_eq!(recovered, secret.to_vec());
    }

    #[test]
    fn test_threshold_below_2_fails() {
        let secret = [1u8; 32];
        let result = split_secret(&secret, 1, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_total_below_threshold_fails() {
        let secret = [1u8; 32];
        let result = split_secret(&secret, 4, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_secret_fails() {
        let result = split_secret(&[], 2, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_reconstruct_insufficient_shares() {
        let result = reconstruct_secret(&[ShamirShare {
            data: vec![1, 2, 3],
        }]);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_share_subsets_produce_same_secret() {
        let secret = b"norn-social-recovery-test-secret";
        let shares = split_secret(secret, 3, 6).unwrap();

        // Any 3 of 6 should work.
        let r1 =
            reconstruct_secret(&[shares[0].clone(), shares[2].clone(), shares[4].clone()]).unwrap();
        let r2 =
            reconstruct_secret(&[shares[1].clone(), shares[3].clone(), shares[5].clone()]).unwrap();
        let r3 =
            reconstruct_secret(&[shares[0].clone(), shares[1].clone(), shares[2].clone()]).unwrap();

        assert_eq!(r1, secret.to_vec());
        assert_eq!(r2, secret.to_vec());
        assert_eq!(r3, secret.to_vec());
    }
}
