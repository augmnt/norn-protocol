use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::Keypair;
use norn_types::fraud::FraudProofSubmission;
use norn_types::network::NornMessage;
use norn_types::primitives::{Address, ThreadId, Timestamp};

use crate::monitor::ThreadMonitor;

/// The spindle service manages thread monitoring and fraud proof generation.
///
/// It processes incoming `NornMessage`s, detects fraud via the `ThreadMonitor`,
/// and produces fraud proof messages for broadcast.
pub struct SpindleService {
    monitor: ThreadMonitor,
    keypair: Keypair,
    address: Address,
    pending_fraud_proofs: Vec<FraudProofSubmission>,
}

impl SpindleService {
    /// Create a new spindle service with the given keypair.
    pub fn new(keypair: Keypair) -> Self {
        let address = pubkey_to_address(&keypair.public_key());
        Self {
            monitor: ThreadMonitor::new(),
            keypair,
            address,
            pending_fraud_proofs: Vec::new(),
        }
    }

    /// Get the address of this spindle.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Start watching a thread for fraud.
    pub fn watch_thread(&mut self, thread_id: ThreadId) {
        self.monitor.watch(thread_id);
    }

    /// Stop watching a thread.
    pub fn unwatch_thread(&mut self, thread_id: &ThreadId) {
        self.monitor.unwatch(thread_id);
    }

    /// Process an incoming network message.
    ///
    /// If the message is a `KnotProposal` or `KnotResponse`, extract the knot
    /// and pass it to the monitor. If fraud is detected, build a fraud proof,
    /// add it to the pending queue, and return a `FraudProof` `NornMessage`.
    pub fn on_message(&mut self, msg: &NornMessage, timestamp: Timestamp) -> Vec<NornMessage> {
        let mut responses = Vec::new();

        let knot = match msg {
            NornMessage::KnotProposal(knot) => Some(knot.as_ref()),
            NornMessage::KnotResponse(knot) => Some(knot.as_ref()),
            _ => None,
        };

        if let Some(knot) = knot {
            if let Some(alert) = self.monitor.on_knot(knot) {
                let submission = ThreadMonitor::build_fraud_proof(
                    &alert,
                    self.keypair.public_key(),
                    timestamp,
                    &self.keypair,
                );

                let fraud_msg = NornMessage::FraudProof(Box::new(submission.clone()));
                self.pending_fraud_proofs.push(submission);
                responses.push(fraud_msg);
            }
        }

        responses
    }

    /// Drain all pending fraud proofs that have been generated.
    pub fn drain_fraud_proofs(&mut self) -> Vec<FraudProofSubmission> {
        std::mem::take(&mut self.pending_fraud_proofs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_thread::knot::compute_knot_id;
    use norn_types::fraud::FraudProof;
    use norn_types::knot::{Knot, KnotPayload, KnotType, ParticipantState, TransferPayload};
    use norn_types::primitives::NATIVE_TOKEN_ID;

    /// Helper: create a test knot for the given thread_id, version, and timestamp.
    fn make_test_knot(thread_id: [u8; 20], version: u64, timestamp: u64) -> Knot {
        let pubkey = [1u8; 32];
        let from_addr = thread_id;
        let to_addr = [2u8; 20];

        let before_state = ParticipantState {
            thread_id,
            pubkey,
            version,
            state_hash: [0u8; 32],
        };

        let after_state = ParticipantState {
            thread_id,
            pubkey,
            version: version + 1,
            state_hash: [1u8; 32],
        };

        let payload = KnotPayload::Transfer(TransferPayload {
            token_id: NATIVE_TOKEN_ID,
            amount: 100,
            from: from_addr,
            to: to_addr,
            memo: None,
        });

        let mut knot = Knot {
            id: [0u8; 32],
            knot_type: KnotType::Transfer,
            timestamp,
            expiry: None,
            before_states: vec![before_state],
            after_states: vec![after_state],
            payload,
            signatures: vec![],
        };

        knot.id = compute_knot_id(&knot);
        knot
    }

    #[test]
    fn test_service_detects_double_knot_from_proposals() {
        let keypair = Keypair::generate();
        let mut service = SpindleService::new(keypair);

        let thread_id = [10u8; 20];
        service.watch_thread(thread_id);

        // First knot proposal at version 5
        let knot_a = make_test_knot(thread_id, 5, 1000);
        let msg_a = NornMessage::KnotProposal(Box::new(knot_a));
        let responses = service.on_message(&msg_a, 1000);
        assert!(
            responses.is_empty(),
            "first proposal should produce no fraud proof"
        );

        // Second knot proposal at same version 5 but different knot
        let knot_b = make_test_knot(thread_id, 5, 2000);
        let msg_b = NornMessage::KnotProposal(Box::new(knot_b));
        let responses = service.on_message(&msg_b, 2000);
        assert_eq!(
            responses.len(),
            1,
            "double knot should produce a fraud proof message"
        );

        // Verify the returned message is a FraudProof.
        match &responses[0] {
            NornMessage::FraudProof(submission) => match &submission.proof {
                FraudProof::DoubleKnot { thread_id: tid, .. } => {
                    assert_eq!(*tid, thread_id);
                }
                _ => panic!("expected DoubleKnot fraud proof"),
            },
            _ => panic!("expected FraudProof message"),
        }

        // Drain should return the pending proof.
        let drained = service.drain_fraud_proofs();
        assert_eq!(drained.len(), 1);

        // After draining, no more pending proofs.
        let drained_again = service.drain_fraud_proofs();
        assert!(drained_again.is_empty());
    }

    #[test]
    fn test_service_detects_from_knot_response() {
        let keypair = Keypair::generate();
        let mut service = SpindleService::new(keypair);

        let thread_id = [20u8; 20];
        service.watch_thread(thread_id);

        // First via KnotProposal
        let knot_a = make_test_knot(thread_id, 3, 1000);
        let msg_a = NornMessage::KnotProposal(Box::new(knot_a));
        assert!(service.on_message(&msg_a, 1000).is_empty());

        // Second via KnotResponse with same version
        let knot_b = make_test_knot(thread_id, 3, 2000);
        let msg_b = NornMessage::KnotResponse(Box::new(knot_b));
        let responses = service.on_message(&msg_b, 2000);
        assert_eq!(responses.len(), 1);
    }

    #[test]
    fn test_service_ignores_non_knot_messages() {
        let keypair = Keypair::generate();
        let mut service = SpindleService::new(keypair);

        let thread_id = [30u8; 20];
        service.watch_thread(thread_id);

        // A non-knot message should produce no responses.
        let msg = NornMessage::Alert(norn_types::network::SpindleAlert {
            from: [0u8; 20],
            subject: [0u8; 20],
            reason: "test".to_string(),
            timestamp: 1000,
            signature: [0u8; 64],
        });
        let responses = service.on_message(&msg, 1000);
        assert!(responses.is_empty());
    }

    #[test]
    fn test_service_no_detection_on_unwatched_thread() {
        let keypair = Keypair::generate();
        let mut service = SpindleService::new(keypair);

        // Do NOT watch any thread.
        let thread_id = [40u8; 20];

        let knot_a = make_test_knot(thread_id, 1, 1000);
        let knot_b = make_test_knot(thread_id, 1, 2000);

        assert!(service
            .on_message(&NornMessage::KnotProposal(Box::new(knot_a)), 1000)
            .is_empty());
        assert!(service
            .on_message(&NornMessage::KnotProposal(Box::new(knot_b)), 2000)
            .is_empty());
    }

    #[test]
    fn test_service_address() {
        let keypair = Keypair::generate();
        let expected_address = pubkey_to_address(&keypair.public_key());
        let service = SpindleService::new(keypair);
        assert_eq!(*service.address(), expected_address);
    }
}
