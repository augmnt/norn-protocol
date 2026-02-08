use std::collections::HashSet;

use norn_crypto::keys::verify;
use norn_types::loom::{
    compute_loom_id, loom_deploy_signing_data, validate_loom_name, LoomRegistration,
};
use norn_types::primitives::LoomId;

use crate::error::WeaveError;

/// Validate a loom registration (deploy).
///
/// Checks: name format, no duplicate loom_id, valid config,
/// operator pubkey-address match, and signature.
/// Returns the computed loom ID on success.
pub fn validate_loom_registration(
    reg: &LoomRegistration,
    known_looms: &HashSet<LoomId>,
) -> Result<LoomId, WeaveError> {
    // 1. Validate name format.
    validate_loom_name(&reg.config.name).map_err(|e| WeaveError::InvalidLoomRegistration {
        reason: e.to_string(),
    })?;

    // 2. Validate config constraints.
    if reg.config.min_participants == 0 {
        return Err(WeaveError::InvalidLoomRegistration {
            reason: "min_participants must be >= 1".to_string(),
        });
    }
    if reg.config.max_participants < reg.config.min_participants {
        return Err(WeaveError::InvalidLoomRegistration {
            reason: format!(
                "max_participants ({}) must be >= min_participants ({})",
                reg.config.max_participants, reg.config.min_participants
            ),
        });
    }

    // 3. Verify signature.
    let sig_data = loom_deploy_signing_data(reg);
    verify(&sig_data, &reg.signature, &reg.operator).map_err(|_| {
        WeaveError::InvalidLoomRegistration {
            reason: "invalid signature".to_string(),
        }
    })?;

    // 4. Compute loom_id and check not duplicate.
    let loom_id = compute_loom_id(reg);
    if known_looms.contains(&loom_id) {
        return Err(WeaveError::InvalidLoomRegistration {
            reason: format!("loom already exists: {}", hex::encode(loom_id)),
        });
    }

    Ok(loom_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::keys::Keypair;

    fn make_signed_loom_registration(kp: &Keypair, name: &str) -> LoomRegistration {
        let mut reg = LoomRegistration {
            config: norn_types::loom::LoomConfig {
                loom_id: [0u8; 32],
                name: name.to_string(),
                max_participants: 100,
                min_participants: 1,
                accepted_tokens: vec![],
                config_data: vec![],
            },
            operator: kp.public_key(),
            timestamp: 1000,
            signature: [0u8; 64],
        };
        let sig_data = loom_deploy_signing_data(&reg);
        reg.signature = kp.sign(&sig_data);
        reg
    }

    #[test]
    fn test_valid_loom_registration() {
        let kp = Keypair::generate();
        let reg = make_signed_loom_registration(&kp, "counter");
        let known = HashSet::new();
        assert!(validate_loom_registration(&reg, &known).is_ok());
    }

    #[test]
    fn test_duplicate_loom_rejected() {
        let kp = Keypair::generate();
        let reg = make_signed_loom_registration(&kp, "counter");
        let loom_id = compute_loom_id(&reg);
        let mut known = HashSet::new();
        known.insert(loom_id);
        assert!(matches!(
            validate_loom_registration(&reg, &known),
            Err(WeaveError::InvalidLoomRegistration { .. })
        ));
    }

    #[test]
    fn test_invalid_name_rejected() {
        let kp = Keypair::generate();
        let reg = make_signed_loom_registration(&kp, ""); // empty name
        let known = HashSet::new();
        assert!(matches!(
            validate_loom_registration(&reg, &known),
            Err(WeaveError::InvalidLoomRegistration { .. })
        ));
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let kp = Keypair::generate();
        let mut reg = make_signed_loom_registration(&kp, "counter");
        reg.signature[0] ^= 0xff;
        let known = HashSet::new();
        assert!(matches!(
            validate_loom_registration(&reg, &known),
            Err(WeaveError::InvalidLoomRegistration { .. })
        ));
    }

    #[test]
    fn test_invalid_config_rejected() {
        let kp = Keypair::generate();
        let mut reg = make_signed_loom_registration(&kp, "counter");
        reg.config.min_participants = 0;
        // Re-sign after mutation.
        let sig_data = loom_deploy_signing_data(&reg);
        reg.signature = kp.sign(&sig_data);
        let known = HashSet::new();
        assert!(matches!(
            validate_loom_registration(&reg, &known),
            Err(WeaveError::InvalidLoomRegistration { .. })
        ));
    }

    #[test]
    fn test_max_less_than_min_rejected() {
        let kp = Keypair::generate();
        let mut reg = make_signed_loom_registration(&kp, "counter");
        reg.config.max_participants = 0;
        reg.config.min_participants = 1;
        let sig_data = loom_deploy_signing_data(&reg);
        reg.signature = kp.sign(&sig_data);
        let known = HashSet::new();
        assert!(matches!(
            validate_loom_registration(&reg, &known),
            Err(WeaveError::InvalidLoomRegistration { .. })
        ));
    }
}
