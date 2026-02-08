use std::collections::{HashMap, HashSet};

use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::verify;
use norn_types::primitives::{Amount, TokenId};
use norn_types::token::{
    compute_token_id, validate_token_name, validate_token_symbol, MAX_TOKEN_DECIMALS,
};
use norn_types::weave::{TokenBurn, TokenDefinition, TokenMint};

use crate::error::WeaveError;

/// Metadata tracked per token in the weave engine.
#[derive(Debug, Clone)]
pub struct TokenMeta {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub max_supply: Amount,
    pub current_supply: Amount,
    pub creator: [u8; 20],
    pub created_at: u64,
}

/// Compute the data that should be signed for a token definition.
pub fn token_definition_signing_data(def: &TokenDefinition) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(def.name.as_bytes());
    data.extend_from_slice(def.symbol.as_bytes());
    data.push(def.decimals);
    data.extend_from_slice(&def.max_supply.to_le_bytes());
    data.extend_from_slice(&def.initial_supply.to_le_bytes());
    data.extend_from_slice(&def.creator);
    data.extend_from_slice(&def.timestamp.to_le_bytes());
    data
}

/// Compute the data that should be signed for a token mint.
pub fn token_mint_signing_data(mint: &TokenMint) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&mint.token_id);
    data.extend_from_slice(&mint.to);
    data.extend_from_slice(&mint.amount.to_le_bytes());
    data.extend_from_slice(&mint.authority);
    data.extend_from_slice(&mint.timestamp.to_le_bytes());
    data
}

/// Compute the data that should be signed for a token burn.
pub fn token_burn_signing_data(burn: &TokenBurn) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&burn.token_id);
    data.extend_from_slice(&burn.burner);
    data.extend_from_slice(&burn.amount.to_le_bytes());
    data.extend_from_slice(&burn.timestamp.to_le_bytes());
    data
}

/// Validate a token definition.
///
/// Returns the computed token ID on success.
pub fn validate_token_definition(
    def: &TokenDefinition,
    known_tokens: &HashMap<TokenId, TokenMeta>,
    known_symbols: &HashSet<String>,
) -> Result<TokenId, WeaveError> {
    // 1. Validate name format.
    validate_token_name(&def.name).map_err(|e| WeaveError::InvalidTokenDefinition {
        reason: e.to_string(),
    })?;

    // 2. Validate symbol format.
    validate_token_symbol(&def.symbol).map_err(|e| WeaveError::InvalidTokenDefinition {
        reason: e.to_string(),
    })?;

    // 3. Decimals <= 18.
    if def.decimals > MAX_TOKEN_DECIMALS {
        return Err(WeaveError::InvalidTokenDefinition {
            reason: format!(
                "decimals must be <= {MAX_TOKEN_DECIMALS}, got {}",
                def.decimals
            ),
        });
    }

    // 4. initial_supply <= max_supply (when max > 0).
    if def.max_supply > 0 && def.initial_supply > def.max_supply {
        return Err(WeaveError::InvalidTokenDefinition {
            reason: format!(
                "initial supply {} exceeds max supply {}",
                def.initial_supply, def.max_supply
            ),
        });
    }

    // 5. pubkey_to_address(creator_pubkey) == creator.
    let expected_address = pubkey_to_address(&def.creator_pubkey);
    if def.creator != expected_address {
        return Err(WeaveError::InvalidTokenDefinition {
            reason: "creator address does not match creator_pubkey".to_string(),
        });
    }

    // 6. Verify signature.
    let sig_data = token_definition_signing_data(def);
    verify(&sig_data, &def.signature, &def.creator_pubkey).map_err(|_| {
        WeaveError::InvalidTokenDefinition {
            reason: "invalid signature".to_string(),
        }
    })?;

    // 7. Compute token_id, check not duplicate.
    let token_id = compute_token_id(
        &def.creator,
        &def.name,
        &def.symbol,
        def.decimals,
        def.max_supply,
        def.timestamp,
    );
    if known_tokens.contains_key(&token_id) {
        return Err(WeaveError::InvalidTokenDefinition {
            reason: format!("token already exists: {}", hex::encode(token_id)),
        });
    }

    // 8. Check symbol not taken.
    if known_symbols.contains(&def.symbol) {
        return Err(WeaveError::InvalidTokenDefinition {
            reason: format!("symbol already taken: {}", def.symbol),
        });
    }

    Ok(token_id)
}

/// Validate a token mint.
pub fn validate_token_mint(
    mint: &TokenMint,
    known_tokens: &HashMap<TokenId, TokenMeta>,
) -> Result<(), WeaveError> {
    // 1. Token exists.
    let meta = known_tokens
        .get(&mint.token_id)
        .ok_or_else(|| WeaveError::InvalidTokenMint {
            reason: format!("token not found: {}", hex::encode(mint.token_id)),
        })?;

    // 2. Authority == token creator.
    if mint.authority != meta.creator {
        return Err(WeaveError::InvalidTokenMint {
            reason: "not token authority".to_string(),
        });
    }

    // 3. Pubkey matches authority.
    let expected_address = pubkey_to_address(&mint.authority_pubkey);
    if mint.authority != expected_address {
        return Err(WeaveError::InvalidTokenMint {
            reason: "authority address does not match authority_pubkey".to_string(),
        });
    }

    // 4. Verify signature.
    let sig_data = token_mint_signing_data(mint);
    verify(&sig_data, &mint.signature, &mint.authority_pubkey).map_err(|_| {
        WeaveError::InvalidTokenMint {
            reason: "invalid signature".to_string(),
        }
    })?;

    // 5. current_supply + amount <= max_supply (when max > 0).
    if meta.max_supply > 0 {
        let new_supply = meta.current_supply.saturating_add(mint.amount);
        if new_supply > meta.max_supply {
            return Err(WeaveError::InvalidTokenMint {
                reason: format!(
                    "supply cap exceeded: {} + {} > {}",
                    meta.current_supply, mint.amount, meta.max_supply
                ),
            });
        }
    }

    // 6. Amount > 0.
    if mint.amount == 0 {
        return Err(WeaveError::InvalidTokenMint {
            reason: "amount must be positive".to_string(),
        });
    }

    Ok(())
}

/// Validate a token burn.
pub fn validate_token_burn(
    burn: &TokenBurn,
    known_tokens: &HashMap<TokenId, TokenMeta>,
) -> Result<(), WeaveError> {
    // 1. Token exists.
    if !known_tokens.contains_key(&burn.token_id) {
        return Err(WeaveError::InvalidTokenBurn {
            reason: format!("token not found: {}", hex::encode(burn.token_id)),
        });
    }

    // 2. Pubkey matches burner.
    let expected_address = pubkey_to_address(&burn.burner_pubkey);
    if burn.burner != expected_address {
        return Err(WeaveError::InvalidTokenBurn {
            reason: "burner address does not match burner_pubkey".to_string(),
        });
    }

    // 3. Verify signature.
    let sig_data = token_burn_signing_data(burn);
    verify(&sig_data, &burn.signature, &burn.burner_pubkey).map_err(|_| {
        WeaveError::InvalidTokenBurn {
            reason: "invalid signature".to_string(),
        }
    })?;

    // 4. Amount > 0.
    if burn.amount == 0 {
        return Err(WeaveError::InvalidTokenBurn {
            reason: "amount must be positive".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::address::pubkey_to_address;
    use norn_crypto::keys::Keypair;
    use norn_types::token::TOKEN_CREATION_FEE;

    fn make_signed_token_definition(kp: &Keypair, name: &str, symbol: &str) -> TokenDefinition {
        let creator = pubkey_to_address(&kp.public_key());
        let mut def = TokenDefinition {
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimals: 8,
            max_supply: 1_000_000,
            initial_supply: 1_000,
            creator,
            creator_pubkey: kp.public_key(),
            timestamp: 1000,
            signature: [0u8; 64],
        };
        let sig_data = token_definition_signing_data(&def);
        def.signature = kp.sign(&sig_data);
        def
    }

    fn make_token_meta(kp: &Keypair, symbol: &str) -> (TokenId, TokenMeta) {
        let creator = pubkey_to_address(&kp.public_key());
        let token_id = compute_token_id(&creator, "Test Token", symbol, 8, 1_000_000, 1000);
        let meta = TokenMeta {
            name: "Test Token".to_string(),
            symbol: symbol.to_string(),
            decimals: 8,
            max_supply: 1_000_000,
            current_supply: 1_000,
            creator,
            created_at: 1000,
        };
        (token_id, meta)
    }

    #[test]
    fn test_valid_token_definition() {
        let kp = Keypair::generate();
        let def = make_signed_token_definition(&kp, "Test Token", "TST");
        let known_tokens = HashMap::new();
        let known_symbols = HashSet::new();
        assert!(validate_token_definition(&def, &known_tokens, &known_symbols).is_ok());
    }

    #[test]
    fn test_duplicate_symbol_rejected() {
        let kp = Keypair::generate();
        let def = make_signed_token_definition(&kp, "Test Token", "TST");
        let known_tokens = HashMap::new();
        let mut known_symbols = HashSet::new();
        known_symbols.insert("TST".to_string());
        assert!(matches!(
            validate_token_definition(&def, &known_tokens, &known_symbols),
            Err(WeaveError::InvalidTokenDefinition { .. })
        ));
    }

    #[test]
    fn test_invalid_symbol_rejected() {
        let kp = Keypair::generate();
        let def = make_signed_token_definition(&kp, "Test Token", "tst"); // lowercase
        let known_tokens = HashMap::new();
        let known_symbols = HashSet::new();
        assert!(matches!(
            validate_token_definition(&def, &known_tokens, &known_symbols),
            Err(WeaveError::InvalidTokenDefinition { .. })
        ));
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let kp = Keypair::generate();
        let mut def = make_signed_token_definition(&kp, "Test Token", "TST");
        def.signature[0] ^= 0xff;
        let known_tokens = HashMap::new();
        let known_symbols = HashSet::new();
        assert!(matches!(
            validate_token_definition(&def, &known_tokens, &known_symbols),
            Err(WeaveError::InvalidTokenDefinition { .. })
        ));
    }

    #[test]
    fn test_initial_supply_exceeds_max_rejected() {
        let kp = Keypair::generate();
        let creator = pubkey_to_address(&kp.public_key());
        let mut def = TokenDefinition {
            name: "Test".to_string(),
            symbol: "TST".to_string(),
            decimals: 8,
            max_supply: 100,
            initial_supply: 200, // exceeds max
            creator,
            creator_pubkey: kp.public_key(),
            timestamp: 1000,
            signature: [0u8; 64],
        };
        let sig_data = token_definition_signing_data(&def);
        def.signature = kp.sign(&sig_data);

        let known_tokens = HashMap::new();
        let known_symbols = HashSet::new();
        assert!(matches!(
            validate_token_definition(&def, &known_tokens, &known_symbols),
            Err(WeaveError::InvalidTokenDefinition { .. })
        ));
    }

    #[test]
    fn test_valid_token_mint() {
        let kp = Keypair::generate();
        let (token_id, meta) = make_token_meta(&kp, "TST");
        let authority = pubkey_to_address(&kp.public_key());

        let mut mint = TokenMint {
            token_id,
            to: [5u8; 20],
            amount: 500,
            authority,
            authority_pubkey: kp.public_key(),
            timestamp: 2000,
            signature: [0u8; 64],
        };
        let sig_data = token_mint_signing_data(&mint);
        mint.signature = kp.sign(&sig_data);

        let mut known_tokens = HashMap::new();
        known_tokens.insert(token_id, meta);
        assert!(validate_token_mint(&mint, &known_tokens).is_ok());
    }

    #[test]
    fn test_mint_non_existent_token_rejected() {
        let kp = Keypair::generate();
        let authority = pubkey_to_address(&kp.public_key());
        let mut mint = TokenMint {
            token_id: [99u8; 32],
            to: [5u8; 20],
            amount: 500,
            authority,
            authority_pubkey: kp.public_key(),
            timestamp: 2000,
            signature: [0u8; 64],
        };
        let sig_data = token_mint_signing_data(&mint);
        mint.signature = kp.sign(&sig_data);

        let known_tokens = HashMap::new();
        assert!(matches!(
            validate_token_mint(&mint, &known_tokens),
            Err(WeaveError::InvalidTokenMint { .. })
        ));
    }

    #[test]
    fn test_mint_not_authority_rejected() {
        let creator_kp = Keypair::generate();
        let other_kp = Keypair::generate();
        let (token_id, meta) = make_token_meta(&creator_kp, "TST");
        let authority = pubkey_to_address(&other_kp.public_key()); // not creator

        let mut mint = TokenMint {
            token_id,
            to: [5u8; 20],
            amount: 500,
            authority,
            authority_pubkey: other_kp.public_key(),
            timestamp: 2000,
            signature: [0u8; 64],
        };
        let sig_data = token_mint_signing_data(&mint);
        mint.signature = other_kp.sign(&sig_data);

        let mut known_tokens = HashMap::new();
        known_tokens.insert(token_id, meta);
        assert!(matches!(
            validate_token_mint(&mint, &known_tokens),
            Err(WeaveError::InvalidTokenMint { .. })
        ));
    }

    #[test]
    fn test_mint_supply_cap_exceeded() {
        let kp = Keypair::generate();
        let (token_id, mut meta) = make_token_meta(&kp, "TST");
        meta.current_supply = 999_999;
        meta.max_supply = 1_000_000;
        let authority = pubkey_to_address(&kp.public_key());

        let mut mint = TokenMint {
            token_id,
            to: [5u8; 20],
            amount: 2, // would exceed cap
            authority,
            authority_pubkey: kp.public_key(),
            timestamp: 2000,
            signature: [0u8; 64],
        };
        let sig_data = token_mint_signing_data(&mint);
        mint.signature = kp.sign(&sig_data);

        let mut known_tokens = HashMap::new();
        known_tokens.insert(token_id, meta);
        assert!(matches!(
            validate_token_mint(&mint, &known_tokens),
            Err(WeaveError::InvalidTokenMint { .. })
        ));
    }

    #[test]
    fn test_valid_token_burn() {
        let kp = Keypair::generate();
        let (token_id, meta) = make_token_meta(&kp, "TST");
        let burner = pubkey_to_address(&kp.public_key());

        let mut burn = TokenBurn {
            token_id,
            burner,
            burner_pubkey: kp.public_key(),
            amount: 100,
            timestamp: 2000,
            signature: [0u8; 64],
        };
        let sig_data = token_burn_signing_data(&burn);
        burn.signature = kp.sign(&sig_data);

        let mut known_tokens = HashMap::new();
        known_tokens.insert(token_id, meta);
        assert!(validate_token_burn(&burn, &known_tokens).is_ok());
    }

    #[test]
    fn test_burn_invalid_signature_rejected() {
        let kp = Keypair::generate();
        let (token_id, meta) = make_token_meta(&kp, "TST");
        let burner = pubkey_to_address(&kp.public_key());

        let mut burn = TokenBurn {
            token_id,
            burner,
            burner_pubkey: kp.public_key(),
            amount: 100,
            timestamp: 2000,
            signature: [0u8; 64],
        };
        let sig_data = token_burn_signing_data(&burn);
        burn.signature = kp.sign(&sig_data);
        burn.signature[0] ^= 0xff;

        let mut known_tokens = HashMap::new();
        known_tokens.insert(token_id, meta);
        assert!(matches!(
            validate_token_burn(&burn, &known_tokens),
            Err(WeaveError::InvalidTokenBurn { .. })
        ));
    }
}
