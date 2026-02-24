use std::collections::HashMap;

use norn_crypto::keys::Keypair;
use norn_thread::knot::compute_knot_id;
use norn_types::fraud::{FraudProof, FraudProofSubmission};
use norn_types::knot::Knot;
use norn_types::primitives::{KnotId, PublicKey, ThreadId, Timestamp, Version};
use norn_types::thread::ThreadHeader;

/// Alert from the monitor about detected issues.
#[derive(Debug, Clone)]
pub enum MonitorAlert {
    DoubleKnot {
        thread_id: ThreadId,
        knot_a: Box<Knot>,
        knot_b: Box<Knot>,
    },
    StaleCommit {
        thread_id: ThreadId,
        commitment: Box<ThreadHeader>,
        expected_version: u64,
        actual_version: u64,
    },
}

/// Maximum number of versions tracked per thread before evicting the oldest.
const MAX_VERSIONS_PER_THREAD: usize = 1_000;

/// Tracks thread state for fraud detection.
struct ThreadWatch {
    thread_id: ThreadId,
    /// Map: version -> Vec<(KnotId, Knot)>
    /// If more than one knot maps to the same version for a given thread, it is a double-knot.
    knots_by_version: HashMap<Version, Vec<(KnotId, Knot)>>,
}

impl ThreadWatch {
    fn new(thread_id: ThreadId) -> Self {
        Self {
            thread_id,
            knots_by_version: HashMap::new(),
        }
    }
}

/// Monitors threads for fraudulent activity such as double-knots.
pub struct ThreadMonitor {
    watched: HashMap<ThreadId, ThreadWatch>,
}

impl ThreadMonitor {
    /// Create a new thread monitor with no watched threads.
    pub fn new() -> Self {
        Self {
            watched: HashMap::new(),
        }
    }

    /// Start watching a thread for fraud.
    pub fn watch(&mut self, thread_id: ThreadId) {
        self.watched
            .entry(thread_id)
            .or_insert_with(|| ThreadWatch::new(thread_id));
    }

    /// Stop watching a thread.
    pub fn unwatch(&mut self, thread_id: &ThreadId) {
        self.watched.remove(thread_id);
    }

    /// Check if a thread is being watched.
    pub fn is_watching(&self, thread_id: &ThreadId) -> bool {
        self.watched.contains_key(thread_id)
    }

    /// Process a new knot and check for fraud.
    ///
    /// For each `before_state` in the knot, if the thread is being watched,
    /// record the knot keyed by `(thread_id, version)`. If there are now 2+
    /// knots for the same key, a double-knot is detected.
    pub fn on_knot(&mut self, knot: &Knot) -> Option<MonitorAlert> {
        let knot_id = compute_knot_id(knot);

        for before_state in &knot.before_states {
            let thread_id = before_state.thread_id;

            if let Some(watch) = self.watched.get_mut(&thread_id) {
                let version = before_state.version;
                let entries = watch
                    .knots_by_version
                    .entry(version)
                    .or_insert_with(Vec::new);

                // Only add if this knot ID isn't already recorded for this version.
                if !entries.iter().any(|(id, _)| *id == knot_id) {
                    entries.push((knot_id, knot.clone()));
                }

                // If there are 2+ different knots at the same version, it is a double-knot.
                if entries.len() >= 2 {
                    return Some(MonitorAlert::DoubleKnot {
                        thread_id: watch.thread_id,
                        knot_a: Box::new(entries[0].1.clone()),
                        knot_b: Box::new(entries[1].1.clone()),
                    });
                }

                // Evict oldest versions if the map grows too large.
                if watch.knots_by_version.len() > MAX_VERSIONS_PER_THREAD {
                    if let Some(&min_version) = watch.knots_by_version.keys().min() {
                        watch.knots_by_version.remove(&min_version);
                    }
                }
            }
        }

        None
    }

    /// Process a new commitment and check for stale commits (version regression).
    ///
    /// Returns `Some(MonitorAlert::StaleCommit{...})` if `header.version < known_version`,
    /// indicating the commitment references a version older than what is already known.
    pub fn on_commitment(
        &mut self,
        thread_id: ThreadId,
        header: &ThreadHeader,
        known_version: u64,
    ) -> Option<MonitorAlert> {
        if !self.is_watching(&thread_id) {
            return None;
        }

        if header.version < known_version {
            return Some(MonitorAlert::StaleCommit {
                thread_id,
                commitment: Box::new(header.clone()),
                expected_version: known_version,
                actual_version: header.version,
            });
        }

        None
    }

    /// Build a `FraudProofSubmission` from a `MonitorAlert`.
    ///
    /// Converts the alert into a `FraudProof`, signs it with the given keypair,
    /// and returns a complete `FraudProofSubmission`.
    pub fn build_fraud_proof(
        alert: &MonitorAlert,
        submitter: PublicKey,
        timestamp: Timestamp,
        keypair: &Keypair,
    ) -> FraudProofSubmission {
        let proof = match alert {
            MonitorAlert::DoubleKnot {
                thread_id,
                knot_a,
                knot_b,
            } => FraudProof::DoubleKnot {
                thread_id: *thread_id,
                knot_a: knot_a.clone(),
                knot_b: knot_b.clone(),
            },
            MonitorAlert::StaleCommit {
                thread_id,
                commitment,
                ..
            } => FraudProof::StaleCommit {
                thread_id: *thread_id,
                commitment: commitment.clone(),
                missing_knots: vec![],
            },
        };

        // Build signing data matching the weave's fraud_proof_signing_data format:
        // borsh(proof) + submitter + timestamp.to_le_bytes()
        let mut sig_data = Vec::new();
        let proof_bytes = borsh::to_vec(&proof).expect("FraudProof serialization should not fail");
        sig_data.extend_from_slice(&proof_bytes);
        sig_data.extend_from_slice(&submitter);
        sig_data.extend_from_slice(&timestamp.to_le_bytes());
        let signature = keypair.sign(&sig_data);

        FraudProofSubmission {
            proof,
            submitter,
            timestamp,
            signature,
        }
    }
}

impl Default for ThreadMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::knot::{KnotPayload, KnotType, ParticipantState, TransferPayload};
    use norn_types::primitives::NATIVE_TOKEN_ID;

    /// Helper: create a test knot with the given thread_id, version, and a unique timestamp
    /// to ensure different knot IDs.
    fn make_test_knot(thread_id: ThreadId, version: Version, timestamp: Timestamp) -> Knot {
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
    fn test_detect_double_knot() {
        let mut monitor = ThreadMonitor::new();
        let thread_id = [10u8; 20];
        monitor.watch(thread_id);

        // First knot at version 5
        let knot_a = make_test_knot(thread_id, 5, 1000);
        let alert = monitor.on_knot(&knot_a);
        assert!(alert.is_none(), "first knot should not trigger alert");

        // Second knot at same version 5 but different timestamp -> different ID
        let knot_b = make_test_knot(thread_id, 5, 2000);
        assert_ne!(
            compute_knot_id(&knot_a),
            compute_knot_id(&knot_b),
            "knots should have different IDs"
        );

        let alert = monitor.on_knot(&knot_b);
        assert!(alert.is_some(), "double knot should be detected");

        match alert.unwrap() {
            MonitorAlert::DoubleKnot { thread_id: tid, .. } => {
                assert_eq!(tid, thread_id);
            }
            _ => panic!("expected DoubleKnot alert"),
        }
    }

    #[test]
    fn test_no_false_positive_sequential_versions() {
        let mut monitor = ThreadMonitor::new();
        let thread_id = [20u8; 20];
        monitor.watch(thread_id);

        // Knot at version 1
        let knot_a = make_test_knot(thread_id, 1, 1000);
        assert!(monitor.on_knot(&knot_a).is_none());

        // Knot at version 2 — different version, should not trigger
        let knot_b = make_test_knot(thread_id, 2, 2000);
        assert!(monitor.on_knot(&knot_b).is_none());

        // Knot at version 3
        let knot_c = make_test_knot(thread_id, 3, 3000);
        assert!(monitor.on_knot(&knot_c).is_none());
    }

    #[test]
    fn test_unwatch_stops_detection() {
        let mut monitor = ThreadMonitor::new();
        let thread_id = [30u8; 20];
        monitor.watch(thread_id);

        // First knot at version 1
        let knot_a = make_test_knot(thread_id, 1, 1000);
        assert!(monitor.on_knot(&knot_a).is_none());

        // Unwatch the thread
        monitor.unwatch(&thread_id);
        assert!(!monitor.is_watching(&thread_id));

        // Second knot at same version 1 — should NOT trigger because unwatched
        let knot_b = make_test_knot(thread_id, 1, 2000);
        assert!(monitor.on_knot(&knot_b).is_none());
    }

    #[test]
    fn test_build_fraud_proof_from_alert() {
        let thread_id = [40u8; 20];
        let knot_a = make_test_knot(thread_id, 5, 1000);
        let knot_b = make_test_knot(thread_id, 5, 2000);

        let alert = MonitorAlert::DoubleKnot {
            thread_id,
            knot_a: Box::new(knot_a),
            knot_b: Box::new(knot_b),
        };

        let keypair = Keypair::generate();
        let submitter = keypair.public_key();
        let timestamp = 5000u64;

        let submission = ThreadMonitor::build_fraud_proof(&alert, submitter, timestamp, &keypair);

        // Verify the submission fields.
        assert_eq!(submission.submitter, submitter);
        assert_eq!(submission.timestamp, timestamp);

        // Verify the signature matches the weave's signing protocol:
        // borsh(proof) + submitter + timestamp.to_le_bytes()
        let mut sig_data = Vec::new();
        let proof_bytes = borsh::to_vec(&submission.proof).expect("serialization should not fail");
        sig_data.extend_from_slice(&proof_bytes);
        sig_data.extend_from_slice(&submitter);
        sig_data.extend_from_slice(&timestamp.to_le_bytes());
        assert!(
            norn_crypto::keys::verify(&sig_data, &submission.signature, &submitter).is_ok(),
            "fraud proof signature should be valid"
        );

        // Verify the proof variant matches.
        match &submission.proof {
            FraudProof::DoubleKnot { thread_id: tid, .. } => {
                assert_eq!(*tid, thread_id);
            }
            _ => panic!("expected DoubleKnot proof"),
        }
    }

    #[test]
    fn test_same_knot_twice_not_double_knot() {
        let mut monitor = ThreadMonitor::new();
        let thread_id = [50u8; 20];
        monitor.watch(thread_id);

        let knot = make_test_knot(thread_id, 1, 1000);
        assert!(monitor.on_knot(&knot).is_none());

        // Same exact knot again — same ID, should not count as double.
        assert!(monitor.on_knot(&knot).is_none());
    }

    /// Helper: create a test ThreadHeader with the given thread_id and version.
    fn make_test_header(thread_id: ThreadId, version: Version) -> ThreadHeader {
        ThreadHeader {
            thread_id,
            owner: [1u8; 32],
            version,
            state_hash: [0u8; 32],
            last_knot_hash: [0u8; 32],
            prev_header_hash: [0u8; 32],
            timestamp: 1000,
            signature: [0u8; 64],
        }
    }

    #[test]
    fn test_stale_commit_detected_for_version_regression() {
        let mut monitor = ThreadMonitor::new();
        let thread_id = [60u8; 20];
        monitor.watch(thread_id);

        // Header at version 3, but known version is 5 — stale.
        let header = make_test_header(thread_id, 3);
        let alert = monitor.on_commitment(thread_id, &header, 5);
        assert!(alert.is_some(), "stale commit should be detected");

        match alert.unwrap() {
            MonitorAlert::StaleCommit {
                thread_id: tid,
                expected_version,
                actual_version,
                ..
            } => {
                assert_eq!(tid, thread_id);
                assert_eq!(expected_version, 5);
                assert_eq!(actual_version, 3);
            }
            _ => panic!("expected StaleCommit alert"),
        }
    }

    #[test]
    fn test_stale_commit_not_triggered_for_valid_commitment() {
        let mut monitor = ThreadMonitor::new();
        let thread_id = [70u8; 20];
        monitor.watch(thread_id);

        // Header at version 5, known version is 5 — not stale (equal).
        let header = make_test_header(thread_id, 5);
        assert!(
            monitor.on_commitment(thread_id, &header, 5).is_none(),
            "equal version should not trigger stale commit"
        );

        // Header at version 7, known version is 5 — not stale (ahead).
        let header = make_test_header(thread_id, 7);
        assert!(
            monitor.on_commitment(thread_id, &header, 5).is_none(),
            "ahead version should not trigger stale commit"
        );

        // Unwatched thread should not trigger either.
        let other_thread = [71u8; 20];
        let header = make_test_header(other_thread, 1);
        assert!(
            monitor.on_commitment(other_thread, &header, 10).is_none(),
            "unwatched thread should not trigger stale commit"
        );
    }

    #[test]
    fn test_build_fraud_proof_handles_stale_commit() {
        let thread_id = [80u8; 20];
        let header = make_test_header(thread_id, 3);

        let alert = MonitorAlert::StaleCommit {
            thread_id,
            commitment: Box::new(header),
            expected_version: 5,
            actual_version: 3,
        };

        let keypair = Keypair::generate();
        let submitter = keypair.public_key();
        let timestamp = 6000u64;

        let submission = ThreadMonitor::build_fraud_proof(&alert, submitter, timestamp, &keypair);

        // Verify the submission fields.
        assert_eq!(submission.submitter, submitter);
        assert_eq!(submission.timestamp, timestamp);

        // Verify the signature.
        let mut sig_data = Vec::new();
        let proof_bytes = borsh::to_vec(&submission.proof).expect("serialization should not fail");
        sig_data.extend_from_slice(&proof_bytes);
        sig_data.extend_from_slice(&submitter);
        sig_data.extend_from_slice(&timestamp.to_le_bytes());
        assert!(
            norn_crypto::keys::verify(&sig_data, &submission.signature, &submitter).is_ok(),
            "fraud proof signature should be valid"
        );

        // Verify the proof variant matches.
        match &submission.proof {
            FraudProof::StaleCommit {
                thread_id: tid,
                missing_knots,
                ..
            } => {
                assert_eq!(*tid, thread_id);
                assert!(
                    missing_knots.is_empty(),
                    "missing_knots should be empty for now"
                );
            }
            _ => panic!("expected StaleCommit proof"),
        }
    }
}
