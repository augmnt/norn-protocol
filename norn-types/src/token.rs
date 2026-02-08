use crate::constants::ONE_NORN;
use crate::error::NornError;
use crate::primitives::{Amount, TokenId};

/// Fee for creating a token (10 NORN, burned).
pub const TOKEN_CREATION_FEE: Amount = 10 * ONE_NORN;

/// Maximum length of a token name.
pub const MAX_TOKEN_NAME_LEN: usize = 64;

/// Maximum length of a token symbol.
pub const MAX_TOKEN_SYMBOL_LEN: usize = 12;

/// Maximum decimals for a token.
pub const MAX_TOKEN_DECIMALS: u8 = 18;

/// Validate a token symbol: uppercase alphanumeric, 1-12 chars.
pub fn validate_token_symbol(symbol: &str) -> Result<(), NornError> {
    if symbol.is_empty() || symbol.len() > MAX_TOKEN_SYMBOL_LEN {
        return Err(NornError::InvalidTokenDefinition(format!(
            "symbol must be 1-{MAX_TOKEN_SYMBOL_LEN} characters, got {}",
            symbol.len()
        )));
    }
    for c in symbol.chars() {
        if !c.is_ascii_uppercase() && !c.is_ascii_digit() {
            return Err(NornError::InvalidTokenDefinition(format!(
                "symbol must be uppercase alphanumeric, found '{c}'"
            )));
        }
    }
    Ok(())
}

/// Validate a token name: printable ASCII, 1-64 chars.
pub fn validate_token_name(name: &str) -> Result<(), NornError> {
    if name.is_empty() || name.len() > MAX_TOKEN_NAME_LEN {
        return Err(NornError::InvalidTokenDefinition(format!(
            "name must be 1-{MAX_TOKEN_NAME_LEN} characters, got {}",
            name.len()
        )));
    }
    for c in name.chars() {
        if !c.is_ascii() || c.is_ascii_control() {
            return Err(NornError::InvalidTokenDefinition(format!(
                "name must be printable ASCII, found '{c}'"
            )));
        }
    }
    Ok(())
}

/// Compute the deterministic token ID from a token definition's fields.
pub fn compute_token_id(
    creator: &[u8; 20],
    name: &str,
    symbol: &str,
    decimals: u8,
    max_supply: Amount,
    timestamp: u64,
) -> TokenId {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(creator);
    hasher.update(name.as_bytes());
    hasher.update(symbol.as_bytes());
    hasher.update(&[decimals]);
    hasher.update(&max_supply.to_le_bytes());
    hasher.update(&timestamp.to_le_bytes());
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_token_symbol_valid() {
        assert!(validate_token_symbol("MTK").is_ok());
        assert!(validate_token_symbol("WBTC").is_ok());
        assert!(validate_token_symbol("A").is_ok());
        assert!(validate_token_symbol("TOKEN123").is_ok());
        assert!(validate_token_symbol("ABCDEFGHIJKL").is_ok()); // 12 chars
    }

    #[test]
    fn test_validate_token_symbol_invalid() {
        assert!(validate_token_symbol("").is_err()); // empty
        assert!(validate_token_symbol("abc").is_err()); // lowercase
        assert!(validate_token_symbol("MTK!").is_err()); // special char
        assert!(validate_token_symbol("MT K").is_err()); // space
        assert!(validate_token_symbol("ABCDEFGHIJKLM").is_err()); // 13 chars
    }

    #[test]
    fn test_validate_token_name_valid() {
        assert!(validate_token_name("My Token").is_ok());
        assert!(validate_token_name("Wrapped Bitcoin").is_ok());
        assert!(validate_token_name("A").is_ok());
        assert!(validate_token_name("Token (v2.0)").is_ok());
    }

    #[test]
    fn test_validate_token_name_invalid() {
        assert!(validate_token_name("").is_err()); // empty
        let long_name = "A".repeat(65);
        assert!(validate_token_name(&long_name).is_err()); // too long
    }

    #[test]
    fn test_compute_token_id_deterministic() {
        let creator = [1u8; 20];
        let id1 = compute_token_id(&creator, "Test", "TST", 8, 1000, 12345);
        let id2 = compute_token_id(&creator, "Test", "TST", 8, 1000, 12345);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_compute_token_id_different_inputs() {
        let creator = [1u8; 20];
        let id1 = compute_token_id(&creator, "Test", "TST", 8, 1000, 12345);
        let id2 = compute_token_id(&creator, "Test", "TST", 8, 1000, 12346); // different timestamp
        assert_ne!(id1, id2);

        let id3 = compute_token_id(&creator, "Test", "TST", 18, 1000, 12345); // different decimals
        assert_ne!(id1, id3);

        let id4 = compute_token_id(&[2u8; 20], "Test", "TST", 8, 1000, 12345); // different creator
        assert_ne!(id1, id4);
    }

    #[test]
    fn test_token_creation_fee() {
        assert_eq!(TOKEN_CREATION_FEE, 10 * ONE_NORN);
    }
}
