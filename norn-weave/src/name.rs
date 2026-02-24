use std::collections::{HashMap, HashSet};

use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::verify;
use norn_types::primitives::Address;
use norn_types::weave::{NameRecordUpdate, NameRegistration, NameTransfer};

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

/// Validate a name transfer.
///
/// Checks: name exists, from is current owner, from_pubkey matches from address,
/// and signature over (name + from + to + timestamp).
pub fn validate_name_transfer(
    transfer: &NameTransfer,
    known_name_owners: &HashMap<String, Address>,
) -> Result<(), WeaveError> {
    // 1. Verify name exists and from is the current owner.
    match known_name_owners.get(&transfer.name) {
        None => {
            return Err(WeaveError::InvalidNameTransfer {
                reason: format!("name '{}' not registered", transfer.name),
            });
        }
        Some(owner) if *owner != transfer.from => {
            return Err(WeaveError::InvalidNameTransfer {
                reason: format!(
                    "'{}' is not owned by 0x{}",
                    transfer.name,
                    hex::encode(transfer.from)
                ),
            });
        }
        _ => {}
    }

    // 2. Verify from_pubkey derives to from address.
    let expected_address = pubkey_to_address(&transfer.from_pubkey);
    if transfer.from != expected_address {
        return Err(WeaveError::InvalidNameTransfer {
            reason: "from address does not match from_pubkey".to_string(),
        });
    }

    // 3. Cannot transfer to self.
    if transfer.from == transfer.to {
        return Err(WeaveError::InvalidNameTransfer {
            reason: "cannot transfer name to self".to_string(),
        });
    }

    // 4. Verify signature.
    let sig_data = name_transfer_signing_data(transfer);
    verify(&sig_data, &transfer.signature, &transfer.from_pubkey).map_err(|_| {
        WeaveError::InvalidNameTransfer {
            reason: "invalid signature".to_string(),
        }
    })?;

    Ok(())
}

/// Compute the data that should be signed for a name transfer.
pub fn name_transfer_signing_data(transfer: &NameTransfer) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(transfer.name.as_bytes());
    data.extend_from_slice(&transfer.from);
    data.extend_from_slice(&transfer.to);
    data.extend_from_slice(&transfer.timestamp.to_le_bytes());
    data
}

/// Validate a name record update.
///
/// Checks: owner matches, key is allowed, value length, signature.
pub fn validate_name_record_update(
    update: &NameRecordUpdate,
    known_name_owners: &HashMap<String, Address>,
) -> Result<(), WeaveError> {
    // 1. Verify name exists and owner matches.
    match known_name_owners.get(&update.name) {
        None => {
            return Err(WeaveError::InvalidNameRecordUpdate {
                reason: format!("name '{}' not registered", update.name),
            });
        }
        Some(owner) if *owner != update.owner => {
            return Err(WeaveError::InvalidNameRecordUpdate {
                reason: format!(
                    "'{}' is not owned by 0x{}",
                    update.name,
                    hex::encode(update.owner)
                ),
            });
        }
        _ => {}
    }

    // 2. Verify owner_pubkey derives to owner address.
    let expected_address = pubkey_to_address(&update.owner_pubkey);
    if update.owner != expected_address {
        return Err(WeaveError::InvalidNameRecordUpdate {
            reason: "owner address does not match owner_pubkey".to_string(),
        });
    }

    // 3. Verify key is in the allowed set.
    if !norn_types::name::ALLOWED_RECORD_KEYS.contains(&update.key.as_str()) {
        return Err(WeaveError::InvalidNameRecordUpdate {
            reason: format!(
                "invalid record key '{}'; allowed: {:?}",
                update.key,
                norn_types::name::ALLOWED_RECORD_KEYS
            ),
        });
    }

    // 4. Verify value length.
    if update.value.len() > norn_types::name::MAX_RECORD_VALUE_LEN {
        return Err(WeaveError::InvalidNameRecordUpdate {
            reason: format!(
                "record value too long: {} > {}",
                update.value.len(),
                norn_types::name::MAX_RECORD_VALUE_LEN
            ),
        });
    }

    // 5. Verify signature.
    let sig_data = name_record_update_signing_data(update);
    verify(&sig_data, &update.signature, &update.owner_pubkey).map_err(|_| {
        WeaveError::InvalidNameRecordUpdate {
            reason: "invalid signature".to_string(),
        }
    })?;

    Ok(())
}

/// Compute the data that should be signed for a name record update.
pub fn name_record_update_signing_data(update: &NameRecordUpdate) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(update.name.as_bytes());
    data.extend_from_slice(update.key.as_bytes());
    data.extend_from_slice(update.value.as_bytes());
    data.extend_from_slice(&update.owner);
    data.extend_from_slice(&update.timestamp.to_le_bytes());
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

    // ─── Name Transfer Tests ─────────────────────────────────────────────────

    fn make_signed_name_transfer(kp: &Keypair, name: &str, to: Address) -> NameTransfer {
        let from = pubkey_to_address(&kp.public_key());
        let mut nt = NameTransfer {
            name: name.to_string(),
            from,
            from_pubkey: kp.public_key(),
            to,
            timestamp: 2000,
            signature: [0u8; 64],
        };
        let sig_data = name_transfer_signing_data(&nt);
        nt.signature = kp.sign(&sig_data);
        nt
    }

    #[test]
    fn test_valid_name_transfer() {
        let kp = Keypair::generate();
        let other_kp = Keypair::generate();
        let from = pubkey_to_address(&kp.public_key());
        let to = pubkey_to_address(&other_kp.public_key());
        let nt = make_signed_name_transfer(&kp, "alice", to);
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), from);
        assert!(validate_name_transfer(&nt, &owners).is_ok());
    }

    #[test]
    fn test_name_transfer_not_owner() {
        let kp = Keypair::generate();
        let other_kp = Keypair::generate();
        let to = pubkey_to_address(&other_kp.public_key());
        let nt = make_signed_name_transfer(&kp, "alice", to);
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), [99u8; 20]); // different owner
        assert!(matches!(
            validate_name_transfer(&nt, &owners),
            Err(WeaveError::InvalidNameTransfer { .. })
        ));
    }

    #[test]
    fn test_name_transfer_not_found() {
        let kp = Keypair::generate();
        let other_kp = Keypair::generate();
        let to = pubkey_to_address(&other_kp.public_key());
        let nt = make_signed_name_transfer(&kp, "alice", to);
        let owners = HashMap::new(); // empty
        assert!(matches!(
            validate_name_transfer(&nt, &owners),
            Err(WeaveError::InvalidNameTransfer { .. })
        ));
    }

    #[test]
    fn test_name_transfer_to_self_rejected() {
        let kp = Keypair::generate();
        let from = pubkey_to_address(&kp.public_key());
        let nt = make_signed_name_transfer(&kp, "alice", from);
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), from);
        assert!(matches!(
            validate_name_transfer(&nt, &owners),
            Err(WeaveError::InvalidNameTransfer { .. })
        ));
    }

    #[test]
    fn test_name_transfer_invalid_signature() {
        let kp = Keypair::generate();
        let other_kp = Keypair::generate();
        let from = pubkey_to_address(&kp.public_key());
        let to = pubkey_to_address(&other_kp.public_key());
        let mut nt = make_signed_name_transfer(&kp, "alice", to);
        nt.signature[0] ^= 0xff;
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), from);
        assert!(matches!(
            validate_name_transfer(&nt, &owners),
            Err(WeaveError::InvalidNameTransfer { .. })
        ));
    }

    // ─── Name Record Update Tests ────────────────────────────────────────────

    fn make_signed_record_update(
        kp: &Keypair,
        name: &str,
        key: &str,
        value: &str,
    ) -> NameRecordUpdate {
        let owner = pubkey_to_address(&kp.public_key());
        let mut nru = NameRecordUpdate {
            name: name.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            owner,
            owner_pubkey: kp.public_key(),
            timestamp: 3000,
            signature: [0u8; 64],
        };
        let sig_data = name_record_update_signing_data(&nru);
        nru.signature = kp.sign(&sig_data);
        nru
    }

    #[test]
    fn test_valid_name_record_update() {
        let kp = Keypair::generate();
        let owner = pubkey_to_address(&kp.public_key());
        let nru = make_signed_record_update(&kp, "alice", "avatar", "https://example.com/pic.png");
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), owner);
        assert!(validate_name_record_update(&nru, &owners).is_ok());
    }

    #[test]
    fn test_record_update_invalid_key() {
        let kp = Keypair::generate();
        let owner = pubkey_to_address(&kp.public_key());
        let nru = make_signed_record_update(&kp, "alice", "phone", "555-1234");
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), owner);
        assert!(matches!(
            validate_name_record_update(&nru, &owners),
            Err(WeaveError::InvalidNameRecordUpdate { .. })
        ));
    }

    #[test]
    fn test_record_update_value_too_long() {
        let kp = Keypair::generate();
        let owner = pubkey_to_address(&kp.public_key());
        let long_value = "x".repeat(norn_types::name::MAX_RECORD_VALUE_LEN + 1);
        let nru = make_signed_record_update(&kp, "alice", "url", &long_value);
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), owner);
        assert!(matches!(
            validate_name_record_update(&nru, &owners),
            Err(WeaveError::InvalidNameRecordUpdate { .. })
        ));
    }

    #[test]
    fn test_record_update_not_owner() {
        let kp = Keypair::generate();
        let nru = make_signed_record_update(&kp, "alice", "avatar", "pic.png");
        let mut owners = HashMap::new();
        owners.insert("alice".to_string(), [99u8; 20]);
        assert!(matches!(
            validate_name_record_update(&nru, &owners),
            Err(WeaveError::InvalidNameRecordUpdate { .. })
        ));
    }
}
