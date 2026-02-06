use borsh::BorshSerialize;
use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::Keypair;
use norn_types::error::NornError;
use norn_types::knot::*;
use norn_types::primitives::*;

use crate::state::compute_state_hash;
use norn_types::thread::ThreadState;

/// Builder for constructing knots step by step.
pub struct KnotBuilder {
    knot_type: KnotType,
    timestamp: Timestamp,
    expiry: Option<Timestamp>,
    before_states: Vec<ParticipantState>,
    after_states: Vec<ParticipantState>,
    payload: Option<KnotPayload>,
}

impl KnotBuilder {
    /// Start building a new transfer knot.
    pub fn transfer(timestamp: Timestamp) -> Self {
        Self {
            knot_type: KnotType::Transfer,
            timestamp,
            expiry: None,
            before_states: Vec::new(),
            after_states: Vec::new(),
            payload: None,
        }
    }

    /// Start building a new multi-transfer knot.
    pub fn multi_transfer(timestamp: Timestamp) -> Self {
        Self {
            knot_type: KnotType::MultiTransfer,
            timestamp,
            expiry: None,
            before_states: Vec::new(),
            after_states: Vec::new(),
            payload: None,
        }
    }

    /// Start building a new loom interaction knot.
    pub fn loom_interaction(timestamp: Timestamp) -> Self {
        Self {
            knot_type: KnotType::LoomInteraction,
            timestamp,
            expiry: None,
            before_states: Vec::new(),
            after_states: Vec::new(),
            payload: None,
        }
    }

    /// Set the expiry timestamp.
    pub fn with_expiry(mut self, expiry: Timestamp) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// Add a participant's before state.
    pub fn add_before_state(
        mut self,
        thread_id: ThreadId,
        pubkey: PublicKey,
        version: Version,
        state: &ThreadState,
    ) -> Self {
        self.before_states.push(ParticipantState {
            thread_id,
            pubkey,
            version,
            state_hash: compute_state_hash(state),
        });
        self
    }

    /// Add a participant's after state.
    pub fn add_after_state(
        mut self,
        thread_id: ThreadId,
        pubkey: PublicKey,
        version: Version,
        state: &ThreadState,
    ) -> Self {
        self.after_states.push(ParticipantState {
            thread_id,
            pubkey,
            version,
            state_hash: compute_state_hash(state),
        });
        self
    }

    /// Set the payload.
    pub fn with_payload(mut self, payload: KnotPayload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Build the knot (without signatures). The knot ID is computed from all fields.
    pub fn build(self) -> Result<Knot, NornError> {
        let payload = self.payload.ok_or(NornError::PayloadInconsistent {
            reason: "payload is required".to_string(),
        })?;

        // Create knot with a placeholder ID first
        let mut knot = Knot {
            id: [0u8; 32],
            knot_type: self.knot_type,
            timestamp: self.timestamp,
            expiry: self.expiry,
            before_states: self.before_states,
            after_states: self.after_states,
            payload,
            signatures: Vec::new(),
        };

        // Compute the knot ID
        knot.id = compute_knot_id(&knot);

        Ok(knot)
    }
}

/// Compute the knot ID by hashing all fields except signatures.
pub fn compute_knot_id(knot: &Knot) -> KnotId {
    // We serialize all fields except signatures
    let mut data = Vec::new();
    knot.knot_type
        .serialize(&mut data)
        .expect("serialization should not fail");
    knot.timestamp
        .serialize(&mut data)
        .expect("serialization should not fail");
    knot.expiry
        .serialize(&mut data)
        .expect("serialization should not fail");
    knot.before_states
        .serialize(&mut data)
        .expect("serialization should not fail");
    knot.after_states
        .serialize(&mut data)
        .expect("serialization should not fail");
    knot.payload
        .serialize(&mut data)
        .expect("serialization should not fail");
    blake3_hash(&data)
}

/// Sign a knot with a keypair. Returns the signature over the knot ID.
pub fn sign_knot(knot: &Knot, keypair: &Keypair) -> Signature {
    keypair.sign(&knot.id)
}

/// Add a signature to a knot.
pub fn add_signature(knot: &mut Knot, signature: Signature) {
    knot.signatures.push(signature);
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::knot::TransferPayload;
    use norn_types::primitives::NATIVE_TOKEN_ID;

    fn make_test_transfer_knot() -> (Knot, Keypair, Keypair) {
        let sender_kp = Keypair::generate();
        let receiver_kp = Keypair::generate();
        let sender_addr = norn_crypto::address::pubkey_to_address(&sender_kp.public_key());
        let receiver_addr = norn_crypto::address::pubkey_to_address(&receiver_kp.public_key());

        let mut sender_state = ThreadState::new();
        sender_state.credit(NATIVE_TOKEN_ID, 1000).unwrap();
        let receiver_state = ThreadState::new();

        let mut sender_after = sender_state.clone();
        let mut receiver_after = receiver_state.clone();
        sender_after.debit(&NATIVE_TOKEN_ID, 500);
        receiver_after.credit(NATIVE_TOKEN_ID, 500).unwrap();

        let knot = KnotBuilder::transfer(1000)
            .add_before_state(sender_addr, sender_kp.public_key(), 0, &sender_state)
            .add_before_state(receiver_addr, receiver_kp.public_key(), 0, &receiver_state)
            .add_after_state(sender_addr, sender_kp.public_key(), 1, &sender_after)
            .add_after_state(receiver_addr, receiver_kp.public_key(), 1, &receiver_after)
            .with_payload(KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 500,
                from: sender_addr,
                to: receiver_addr,
                memo: None,
            }))
            .build()
            .unwrap();

        (knot, sender_kp, receiver_kp)
    }

    #[test]
    fn test_knot_builder_creates_valid_knot() {
        let (knot, _, _) = make_test_transfer_knot();
        assert_ne!(knot.id, [0u8; 32]);
        assert_eq!(knot.before_states.len(), 2);
        assert_eq!(knot.after_states.len(), 2);
        assert!(knot.signatures.is_empty());
    }

    #[test]
    fn test_knot_id_is_deterministic() {
        let (knot, _, _) = make_test_transfer_knot();
        let recomputed = compute_knot_id(&knot);
        assert_eq!(knot.id, recomputed);
    }

    #[test]
    fn test_sign_knot() {
        let (knot, sender_kp, _) = make_test_transfer_knot();
        let sig = sign_knot(&knot, &sender_kp);
        assert!(norn_crypto::keys::verify(&knot.id, &sig, &sender_kp.public_key()).is_ok());
    }

    #[test]
    fn test_add_signature() {
        let (mut knot, sender_kp, receiver_kp) = make_test_transfer_knot();
        let sig1 = sign_knot(&knot, &sender_kp);
        let sig2 = sign_knot(&knot, &receiver_kp);
        add_signature(&mut knot, sig1);
        add_signature(&mut knot, sig2);
        assert_eq!(knot.signatures.len(), 2);
    }

    #[test]
    fn test_knot_with_expiry() {
        let sender_kp = Keypair::generate();
        let sender_addr = norn_crypto::address::pubkey_to_address(&sender_kp.public_key());
        let state = ThreadState::new();

        let knot = KnotBuilder::transfer(1000)
            .with_expiry(2000)
            .add_before_state(sender_addr, sender_kp.public_key(), 0, &state)
            .add_after_state(sender_addr, sender_kp.public_key(), 1, &state)
            .with_payload(KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: sender_addr,
                to: [2u8; 20],
                memo: None,
            }))
            .build()
            .unwrap();

        assert_eq!(knot.expiry, Some(2000));
    }

    #[test]
    fn test_builder_without_payload_fails() {
        let result = KnotBuilder::transfer(1000).build();
        assert!(result.is_err());
    }
}
