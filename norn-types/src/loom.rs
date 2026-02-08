use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::constants::ONE_NORN;
use crate::primitives::*;

/// Fee for deploying a loom (50 NORN, burned).
/// Higher than token creation (10 NORN) because bytecode is resource-heavy.
pub const LOOM_DEPLOY_FEE: Amount = 50 * ONE_NORN;

/// Maximum length of a loom name.
pub const MAX_LOOM_NAME_LEN: usize = 64;

/// Compute the data that should be signed for a loom deployment.
/// Canonical bytes: name + operator + timestamp.
pub fn loom_deploy_signing_data(reg: &LoomRegistration) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(reg.config.name.as_bytes());
    data.extend_from_slice(&reg.operator);
    data.extend_from_slice(&reg.timestamp.to_le_bytes());
    data
}

/// Validate a loom name: printable ASCII, 1-64 chars.
pub fn validate_loom_name(name: &str) -> Result<(), crate::error::NornError> {
    if name.is_empty() || name.len() > MAX_LOOM_NAME_LEN {
        return Err(crate::error::NornError::InvalidName(format!(
            "loom name must be 1-{MAX_LOOM_NAME_LEN} characters, got {}",
            name.len()
        )));
    }
    for c in name.chars() {
        if !c.is_ascii() || c.is_ascii_control() {
            return Err(crate::error::NornError::InvalidName(format!(
                "loom name must be printable ASCII, found '{c}'"
            )));
        }
    }
    Ok(())
}

/// Compute a deterministic loom ID from a registration's fields.
pub fn compute_loom_id(reg: &LoomRegistration) -> LoomId {
    use blake3::Hasher;
    let mut hasher = Hasher::new();
    hasher.update(reg.config.name.as_bytes());
    hasher.update(&reg.operator);
    hasher.update(&reg.timestamp.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Configuration for a loom.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomConfig {
    /// Unique identifier for this loom.
    pub loom_id: LoomId,
    /// Human-readable name.
    pub name: String,
    /// Maximum number of participants.
    pub max_participants: usize,
    /// Minimum number of participants for the loom to be active.
    pub min_participants: usize,
    /// Tokens accepted by this loom.
    pub accepted_tokens: Vec<TokenId>,
    /// Opaque loom-specific configuration data.
    pub config_data: Vec<u8>,
}

/// A participant in a loom.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Participant {
    /// Participant's public key.
    pub pubkey: PublicKey,
    /// Participant's address (thread ID).
    pub address: Address,
    /// Timestamp when the participant joined.
    pub joined_at: Timestamp,
    /// Whether the participant is currently active.
    pub active: bool,
}

/// A loom registration request.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomRegistration {
    /// The loom configuration.
    pub config: LoomConfig,
    /// The loom operator's public key.
    pub operator: PublicKey,
    /// Timestamp of registration.
    pub timestamp: Timestamp,
    /// Signature by the operator.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

/// A loom instance with its current state.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Loom {
    /// Loom configuration.
    pub config: LoomConfig,
    /// Loom operator's public key.
    pub operator: PublicKey,
    /// Current participants.
    pub participants: Vec<Participant>,
    /// Hash of the current loom state.
    pub state_hash: Hash,
    /// Current loom state version.
    pub version: Version,
    /// Whether the loom is currently active.
    pub active: bool,
    /// Timestamp of last state update.
    pub last_updated: Timestamp,
}

/// Deployed loom bytecode (Wasm module).
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomBytecode {
    /// The loom this bytecode belongs to.
    pub loom_id: LoomId,
    /// Hash of the Wasm bytecode.
    pub wasm_hash: Hash,
    /// The Wasm bytecode itself.
    pub bytecode: Vec<u8>,
}

/// A loom state transition record.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomStateTransition {
    /// The loom ID.
    pub loom_id: LoomId,
    /// Hash of the state before the transition.
    pub prev_state_hash: Hash,
    /// Hash of the state after the transition.
    pub new_state_hash: Hash,
    /// Inputs to the transition.
    pub inputs: Vec<u8>,
    /// Outputs of the transition.
    pub outputs: Vec<u8>,
}

/// A challenge to a loom state transition.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LoomChallenge {
    /// The loom ID.
    pub loom_id: LoomId,
    /// The disputed state transition.
    pub transition: LoomStateTransition,
    /// The challenger's public key.
    pub challenger: PublicKey,
    /// Timestamp of the challenge.
    pub timestamp: Timestamp,
    /// Signature by the challenger.
    #[serde(with = "crate::primitives::serde_sig")]
    pub signature: Signature,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registration() -> LoomRegistration {
        LoomRegistration {
            config: LoomConfig {
                loom_id: [0u8; 32],
                name: "test-loom".to_string(),
                max_participants: 10,
                min_participants: 1,
                accepted_tokens: vec![],
                config_data: vec![],
            },
            operator: [1u8; 32],
            timestamp: 12345,
            signature: [0u8; 64],
        }
    }

    #[test]
    fn test_loom_deploy_fee() {
        assert_eq!(LOOM_DEPLOY_FEE, 50 * ONE_NORN);
    }

    #[test]
    fn test_loom_deploy_signing_data_deterministic() {
        let reg = make_registration();
        let data1 = loom_deploy_signing_data(&reg);
        let data2 = loom_deploy_signing_data(&reg);
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_compute_loom_id_deterministic() {
        let reg = make_registration();
        let id1 = compute_loom_id(&reg);
        let id2 = compute_loom_id(&reg);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_compute_loom_id_different_inputs() {
        let reg1 = make_registration();
        let mut reg2 = make_registration();
        reg2.timestamp = 99999;
        assert_ne!(compute_loom_id(&reg1), compute_loom_id(&reg2));
    }

    #[test]
    fn test_validate_loom_name_valid() {
        assert!(validate_loom_name("counter").is_ok());
        assert!(validate_loom_name("My Loom (v2)").is_ok());
        assert!(validate_loom_name("A").is_ok());
    }

    #[test]
    fn test_validate_loom_name_invalid() {
        assert!(validate_loom_name("").is_err());
        let long_name = "A".repeat(65);
        assert!(validate_loom_name(&long_name).is_err());
    }
}
