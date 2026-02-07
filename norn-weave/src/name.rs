use std::collections::HashSet;

use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::verify;
use norn_types::weave::NameRegistration;

use crate::error::WeaveError;

/// Validate a name registration.
///
/// Checks: name format, no duplicate, signature over (name + owner + timestamp + fee_paid),
/// and owner_pubkey matches owner address.
pub fn validate_name_registration(
    name_reg: &NameRegistration,
    existing_names: &HashSet<String>,
) -> Result<(), WeaveError> {
    // 1. Validate name format.
    norn_types::name::validate_name(&name_reg.name).map_err(|e| {
        WeaveError::InvalidNameRegistration {
            reason: e.to_string(),
        }
    })?;

    // 2. Check not duplicate.
    if existing_names.contains(&name_reg.name) {
        return Err(WeaveError::DuplicateName {
            name: name_reg.name.clone(),
        });
    }

    // 3. Verify pubkey_to_address(owner_pubkey) == owner.
    let expected_address = pubkey_to_address(&name_reg.owner_pubkey);
    if name_reg.owner != expected_address {
        return Err(WeaveError::InvalidNameRegistration {
            reason: "owner address does not match owner_pubkey".to_string(),
        });
    }

    // 4. Verify signature over (name + owner + timestamp + fee_paid).
    let sig_data = name_registration_signing_data(name_reg);
    verify(&sig_data, &name_reg.signature, &name_reg.owner_pubkey).map_err(|_| {
        WeaveError::InvalidNameRegistration {
            reason: "invalid signature".to_string(),
        }
    })?;

    Ok(())
}

/// Compute the data that should be signed for a name registration.
pub fn name_registration_signing_data(name_reg: &NameRegistration) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(name_reg.name.as_bytes());
    data.extend_from_slice(&name_reg.owner);
    data.extend_from_slice(&name_reg.timestamp.to_le_bytes());
    data.extend_from_slice(&name_reg.fee_paid.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::address::pubkey_to_address;
    use norn_crypto::keys::Keypair;
    use norn_types::name::NAME_REGISTRATION_FEE;

    fn make_signed_name_registration(kp: &Keypair, name: &str) -> NameRegistration {
        let owner = pubkey_to_address(&kp.public_key());
        let mut nr = NameRegistration {
            name: name.to_string(),
            owner,
            owner_pubkey: kp.public_key(),
            timestamp: 1000,
            fee_paid: NAME_REGISTRATION_FEE,
            signature: [0u8; 64],
        };
        let sig_data = name_registration_signing_data(&nr);
        nr.signature = kp.sign(&sig_data);
        nr
    }

    #[test]
    fn test_valid_name_registration() {
        let kp = Keypair::generate();
        let nr = make_signed_name_registration(&kp, "test-name");
        let existing = HashSet::new();
        assert!(validate_name_registration(&nr, &existing).is_ok());
    }

    #[test]
    fn test_duplicate_name_rejected() {
        let kp = Keypair::generate();
        let nr = make_signed_name_registration(&kp, "test-name");
        let mut existing = HashSet::new();
        existing.insert("test-name".to_string());
        assert!(matches!(
            validate_name_registration(&nr, &existing),
            Err(WeaveError::DuplicateName { .. })
        ));
    }

    #[test]
    fn test_invalid_name_format_rejected() {
        let kp = Keypair::generate();
        let nr = make_signed_name_registration(&kp, "ab"); // too short
        let existing = HashSet::new();
        assert!(matches!(
            validate_name_registration(&nr, &existing),
            Err(WeaveError::InvalidNameRegistration { .. })
        ));
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let kp = Keypair::generate();
        let mut nr = make_signed_name_registration(&kp, "test-name");
        nr.signature[0] ^= 0xff;
        let existing = HashSet::new();
        assert!(matches!(
            validate_name_registration(&nr, &existing),
            Err(WeaveError::InvalidNameRegistration { .. })
        ));
    }

    #[test]
    fn test_owner_pubkey_mismatch_rejected() {
        let kp = Keypair::generate();
        let other_kp = Keypair::generate();
        let mut nr = make_signed_name_registration(&kp, "test-name");
        nr.owner = pubkey_to_address(&other_kp.public_key()); // wrong owner
        let existing = HashSet::new();
        assert!(matches!(
            validate_name_registration(&nr, &existing),
            Err(WeaveError::InvalidNameRegistration { .. })
        ));
    }
}
