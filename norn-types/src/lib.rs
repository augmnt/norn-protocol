pub mod consensus;
pub mod constants;
pub mod error;
pub mod fraud;
pub mod genesis;
pub mod knot;
pub mod loom;
pub mod network;
pub mod primitives;
pub mod thread;
pub mod weave;

#[cfg(test)]
mod tests {
    use borsh::{BorshDeserialize, BorshSerialize};

    /// Helper: borsh round-trip test.
    fn borsh_roundtrip<T: BorshSerialize + BorshDeserialize + PartialEq + std::fmt::Debug>(
        value: &T,
    ) {
        let encoded = borsh::to_vec(value).expect("borsh serialize failed");
        let decoded = T::try_from_slice(&encoded).expect("borsh deserialize failed");
        assert_eq!(*value, decoded);
    }

    #[test]
    fn test_thread_header_roundtrip() {
        use crate::thread::ThreadHeader;
        let header = ThreadHeader {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            version: 42,
            state_hash: [3u8; 32],
            last_knot_hash: [0u8; 32],
            prev_header_hash: [4u8; 32],
            timestamp: 1000,
            signature: [5u8; 64],
        };
        borsh_roundtrip(&header);
    }

    #[test]
    fn test_thread_state_roundtrip() {
        use crate::primitives::NATIVE_TOKEN_ID;
        use crate::thread::ThreadState;
        let mut state = ThreadState::new();
        state.credit(NATIVE_TOKEN_ID, 1_000_000).unwrap();
        state.credit([1u8; 32], 500).unwrap();
        state.looms.insert([9u8; 32], vec![1, 2, 3]);
        borsh_roundtrip(&state);
    }

    #[test]
    fn test_knot_roundtrip() {
        use crate::knot::*;
        use crate::primitives::NATIVE_TOKEN_ID;
        let knot = Knot {
            id: [10u8; 32],
            knot_type: KnotType::Transfer,
            timestamp: 2000,
            expiry: Some(5000),
            before_states: vec![ParticipantState {
                thread_id: [1u8; 20],
                pubkey: [2u8; 32],
                version: 0,
                state_hash: [3u8; 32],
            }],
            after_states: vec![ParticipantState {
                thread_id: [1u8; 20],
                pubkey: [2u8; 32],
                version: 1,
                state_hash: [4u8; 32],
            }],
            payload: KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: [1u8; 20],
                to: [2u8; 20],
                memo: Some(b"hello".to_vec()),
            }),
            signatures: vec![[99u8; 64]],
        };
        borsh_roundtrip(&knot);
    }

    #[test]
    fn test_multi_transfer_payload_roundtrip() {
        use crate::knot::*;
        use crate::primitives::NATIVE_TOKEN_ID;
        let payload = MultiTransferPayload {
            transfers: vec![
                TransferPayload {
                    token_id: NATIVE_TOKEN_ID,
                    amount: 100,
                    from: [1u8; 20],
                    to: [2u8; 20],
                    memo: None,
                },
                TransferPayload {
                    token_id: [7u8; 32],
                    amount: 200,
                    from: [3u8; 20],
                    to: [4u8; 20],
                    memo: Some(b"test".to_vec()),
                },
            ],
        };
        borsh_roundtrip(&payload);
    }

    #[test]
    fn test_loom_interaction_payload_roundtrip() {
        use crate::knot::*;
        use crate::primitives::NATIVE_TOKEN_ID;
        let payload = LoomInteractionPayload {
            loom_id: [5u8; 32],
            interaction_type: LoomInteractionType::Deposit,
            token_id: Some(NATIVE_TOKEN_ID),
            amount: Some(1000),
            data: vec![1, 2, 3, 4],
        };
        borsh_roundtrip(&payload);
    }

    #[test]
    fn test_commitment_update_roundtrip() {
        use crate::weave::CommitmentUpdate;
        let cu = CommitmentUpdate {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            version: 10,
            state_hash: [3u8; 32],
            prev_commitment_hash: [4u8; 32],
            knot_count: 5,
            timestamp: 3000,
            signature: [6u8; 64],
        };
        borsh_roundtrip(&cu);
    }

    #[test]
    fn test_registration_roundtrip() {
        use crate::weave::Registration;
        let reg = Registration {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            initial_state_hash: [3u8; 32],
            timestamp: 1000,
            signature: [4u8; 64],
        };
        borsh_roundtrip(&reg);
    }

    #[test]
    fn test_loom_anchor_roundtrip() {
        use crate::weave::LoomAnchor;
        let anchor = LoomAnchor {
            loom_id: [1u8; 32],
            state_hash: [2u8; 32],
            block_height: 100,
            timestamp: 2000,
            signature: [3u8; 64],
        };
        borsh_roundtrip(&anchor);
    }

    #[test]
    fn test_validator_signature_roundtrip() {
        use crate::weave::ValidatorSignature;
        let vs = ValidatorSignature {
            validator: [1u8; 32],
            signature: [2u8; 64],
        };
        borsh_roundtrip(&vs);
    }

    #[test]
    fn test_validator_roundtrip() {
        use crate::weave::Validator;
        let v = Validator {
            pubkey: [1u8; 32],
            address: [2u8; 20],
            stake: 1_000_000,
            active: true,
        };
        borsh_roundtrip(&v);
    }

    #[test]
    fn test_fee_state_roundtrip() {
        use crate::weave::FeeState;
        let fs = FeeState {
            base_fee: 100,
            fee_multiplier: 1000,
            epoch_fees: 50000,
        };
        borsh_roundtrip(&fs);
    }

    #[test]
    fn test_weave_state_roundtrip() {
        use crate::weave::{FeeState, WeaveState};
        let ws = WeaveState {
            height: 100,
            latest_hash: [1u8; 32],
            threads_root: [2u8; 32],
            thread_count: 50,
            fee_state: FeeState {
                base_fee: 100,
                fee_multiplier: 1000,
                epoch_fees: 50000,
            },
        };
        borsh_roundtrip(&ws);
    }

    #[test]
    fn test_fraud_proof_double_knot_roundtrip() {
        use crate::fraud::FraudProof;
        use crate::knot::*;
        use crate::primitives::NATIVE_TOKEN_ID;

        let make_knot = |id_byte: u8| Knot {
            id: [id_byte; 32],
            knot_type: KnotType::Transfer,
            timestamp: 1000,
            expiry: None,
            before_states: vec![],
            after_states: vec![],
            payload: KnotPayload::Transfer(TransferPayload {
                token_id: NATIVE_TOKEN_ID,
                amount: 100,
                from: [1u8; 20],
                to: [2u8; 20],
                memo: None,
            }),
            signatures: vec![],
        };

        let fp = FraudProof::DoubleKnot {
            thread_id: [1u8; 20],
            knot_a: Box::new(make_knot(1)),
            knot_b: Box::new(make_knot(2)),
        };
        borsh_roundtrip(&fp);
    }

    #[test]
    fn test_loom_config_roundtrip() {
        use crate::loom::LoomConfig;
        use crate::primitives::NATIVE_TOKEN_ID;
        let config = LoomConfig {
            loom_id: [1u8; 32],
            name: "Test Loom".to_string(),
            max_participants: 100,
            min_participants: 2,
            accepted_tokens: vec![NATIVE_TOKEN_ID],
            config_data: vec![1, 2, 3],
        };
        borsh_roundtrip(&config);
    }

    #[test]
    fn test_participant_roundtrip() {
        use crate::loom::Participant;
        let p = Participant {
            pubkey: [1u8; 32],
            address: [2u8; 20],
            joined_at: 1000,
            active: true,
        };
        borsh_roundtrip(&p);
    }

    #[test]
    fn test_loom_roundtrip() {
        use crate::loom::{Loom, LoomConfig};
        use crate::primitives::NATIVE_TOKEN_ID;
        let loom = Loom {
            config: LoomConfig {
                loom_id: [1u8; 32],
                name: "Test".to_string(),
                max_participants: 10,
                min_participants: 2,
                accepted_tokens: vec![NATIVE_TOKEN_ID],
                config_data: vec![],
            },
            operator: [2u8; 32],
            participants: vec![],
            state_hash: [3u8; 32],
            version: 0,
            active: true,
            last_updated: 1000,
        };
        borsh_roundtrip(&loom);
    }

    #[test]
    fn test_relay_message_roundtrip() {
        use crate::network::RelayMessage;
        let msg = RelayMessage {
            from: [1u8; 20],
            to: [2u8; 20],
            payload: vec![1, 2, 3, 4],
            timestamp: 1000,
            signature: [3u8; 64],
        };
        borsh_roundtrip(&msg);
    }

    #[test]
    fn test_spindle_registration_roundtrip() {
        use crate::network::SpindleRegistration;
        let sr = SpindleRegistration {
            pubkey: [1u8; 32],
            address: [2u8; 20],
            relay_endpoint: "127.0.0.1:9740".to_string(),
            timestamp: 1000,
            signature: [3u8; 64],
        };
        borsh_roundtrip(&sr);
    }

    #[test]
    fn test_norn_message_roundtrip() {
        use crate::network::NornMessage;
        use crate::weave::Registration;
        let msg = NornMessage::Registration(Registration {
            thread_id: [1u8; 20],
            owner: [2u8; 32],
            initial_state_hash: [3u8; 32],
            timestamp: 1000,
            signature: [4u8; 64],
        });
        borsh_roundtrip(&msg);
    }

    #[test]
    fn test_signed_amount_roundtrip() {
        use crate::primitives::SignedAmount;
        borsh_roundtrip(&SignedAmount::zero());
        borsh_roundtrip(&SignedAmount::positive(42));
        borsh_roundtrip(&SignedAmount::negative(100));
    }

    #[test]
    fn test_knot_type_roundtrip() {
        use crate::knot::KnotType;
        borsh_roundtrip(&KnotType::Transfer);
        borsh_roundtrip(&KnotType::MultiTransfer);
        borsh_roundtrip(&KnotType::LoomInteraction);
    }
}
