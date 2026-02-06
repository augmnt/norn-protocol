use norn_crypto::address::pubkey_to_address;
use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::Keypair;
use norn_types::constants::MAX_UNCOMMITTED_KNOTS;
use norn_types::error::NornError;
use norn_types::knot::Knot;
use norn_types::primitives::*;
use norn_types::thread::{ThreadHeader, ThreadState};

use crate::state::compute_state_hash;

/// A thread with its full state, knot history since last commitment, and header.
pub struct Thread {
    /// The thread owner's keypair.
    keypair: Keypair,
    /// The thread's address (derived from public key).
    address: Address,
    /// Current committed header.
    header: ThreadHeader,
    /// Current mutable state.
    state: ThreadState,
    /// Current version counter.
    version: Version,
    /// Knots accumulated since the last commitment.
    uncommitted_knots: Vec<Knot>,
    /// Timestamp of the last knot applied.
    last_knot_timestamp: Timestamp,
}

impl Thread {
    /// Create a new thread from a keypair. The thread starts with an empty state
    /// and a genesis header.
    pub fn new(keypair: Keypair, timestamp: Timestamp) -> Self {
        let pubkey = keypair.public_key();
        let address = pubkey_to_address(&pubkey);
        let state = ThreadState::new();
        let state_hash = compute_state_hash(&state);

        let header = ThreadHeader {
            thread_id: address,
            owner: pubkey,
            version: 0,
            state_hash,
            last_knot_hash: [0u8; 32],
            prev_header_hash: [0u8; 32],
            timestamp,
            signature: [0u8; 64], // Genesis header has no meaningful signature
        };

        Self {
            keypair,
            address,
            header,
            state,
            version: 0,
            uncommitted_knots: Vec::new(),
            last_knot_timestamp: 0,
        }
    }

    /// Get the thread's address (thread ID).
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Get the thread's public key.
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public_key()
    }

    /// Get a reference to the keypair.
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Get the current committed header.
    pub fn current_header(&self) -> &ThreadHeader {
        &self.header
    }

    /// Get the current state.
    pub fn current_state(&self) -> &ThreadState {
        &self.state
    }

    /// Get a mutable reference to the current state.
    pub fn current_state_mut(&mut self) -> &mut ThreadState {
        &mut self.state
    }

    /// Get the current version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get the timestamp of the last applied knot.
    pub fn last_knot_timestamp(&self) -> Timestamp {
        self.last_knot_timestamp
    }

    /// Get the number of uncommitted knots.
    pub fn uncommitted_count(&self) -> usize {
        self.uncommitted_knots.len()
    }

    /// Apply a fully-signed knot to this thread.
    /// The caller is responsible for validation; this method just updates state.
    pub fn apply_knot(&mut self, knot: Knot, new_state: ThreadState) -> Result<(), NornError> {
        if self.uncommitted_knots.len() >= MAX_UNCOMMITTED_KNOTS {
            return Err(NornError::TooManyUncommittedKnots {
                count: self.uncommitted_knots.len(),
                max: MAX_UNCOMMITTED_KNOTS,
            });
        }

        self.version += 1;
        self.last_knot_timestamp = knot.timestamp;
        self.state = new_state;
        self.uncommitted_knots.push(knot);
        Ok(())
    }

    /// Create a commitment — a new header reflecting the current state.
    /// This "checkpoints" the thread, clearing the uncommitted knot buffer.
    pub fn commit(&mut self, timestamp: Timestamp) -> ThreadHeader {
        let state_hash = compute_state_hash(&self.state);
        let prev_hash = compute_header_hash(&self.header);

        let last_knot_hash = self
            .uncommitted_knots
            .last()
            .map(|k| k.id)
            .unwrap_or([0u8; 32]);

        let mut header = ThreadHeader {
            thread_id: self.address,
            owner: self.keypair.public_key(),
            version: self.version,
            state_hash,
            last_knot_hash,
            prev_header_hash: prev_hash,
            timestamp,
            signature: [0u8; 64],
        };

        // Sign the header
        let header_bytes = header_signing_bytes(&header);
        header.signature = self.keypair.sign(&header_bytes);

        self.header = header.clone();
        self.uncommitted_knots.clear();

        header
    }

    /// Get the uncommitted knots.
    pub fn uncommitted_knots(&self) -> &[Knot] {
        &self.uncommitted_knots
    }
}

/// Compute the hash of a thread header for chaining.
pub fn compute_header_hash(header: &ThreadHeader) -> Hash {
    let bytes = borsh::to_vec(header).expect("header serialization should not fail");
    blake3_hash(&bytes)
}

/// Get the bytes to sign for a thread header (all fields except signature).
fn header_signing_bytes(header: &ThreadHeader) -> Vec<u8> {
    use borsh::BorshSerialize;
    let mut data = Vec::new();
    header.thread_id.serialize(&mut data).unwrap();
    header.owner.serialize(&mut data).unwrap();
    header.version.serialize(&mut data).unwrap();
    header.state_hash.serialize(&mut data).unwrap();
    header.last_knot_hash.serialize(&mut data).unwrap();
    header.prev_header_hash.serialize(&mut data).unwrap();
    header.timestamp.serialize(&mut data).unwrap();
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::primitives::NATIVE_TOKEN_ID;

    #[test]
    fn test_new_thread() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let thread = Thread::new(kp, 1000);

        assert_eq!(thread.version(), 0);
        assert_eq!(thread.public_key(), pk);
        assert_eq!(thread.uncommitted_count(), 0);
        assert_eq!(thread.current_state().balances.len(), 0);
    }

    #[test]
    fn test_thread_address_matches_pubkey() {
        let kp = Keypair::generate();
        let expected_addr = pubkey_to_address(&kp.public_key());
        let thread = Thread::new(kp, 1000);
        assert_eq!(*thread.address(), expected_addr);
    }

    #[test]
    fn test_apply_knot_increments_version() {
        let kp = Keypair::generate();
        let mut thread = Thread::new(kp, 1000);

        let dummy_knot = crate::knot::KnotBuilder::transfer(1001)
            .with_payload(norn_types::knot::KnotPayload::Transfer(
                norn_types::knot::TransferPayload {
                    token_id: NATIVE_TOKEN_ID,
                    amount: 100,
                    from: *thread.address(),
                    to: [2u8; 20],
                    memo: None,
                },
            ))
            .build()
            .unwrap();

        let new_state = ThreadState::new();
        thread.apply_knot(dummy_knot, new_state).unwrap();
        assert_eq!(thread.version(), 1);
        assert_eq!(thread.uncommitted_count(), 1);
    }

    #[test]
    fn test_commit_clears_uncommitted() {
        let kp = Keypair::generate();
        let mut thread = Thread::new(kp, 1000);

        let dummy_knot = crate::knot::KnotBuilder::transfer(1001)
            .with_payload(norn_types::knot::KnotPayload::Transfer(
                norn_types::knot::TransferPayload {
                    token_id: NATIVE_TOKEN_ID,
                    amount: 100,
                    from: *thread.address(),
                    to: [2u8; 20],
                    memo: None,
                },
            ))
            .build()
            .unwrap();

        let new_state = ThreadState::new();
        thread.apply_knot(dummy_knot, new_state).unwrap();
        assert_eq!(thread.uncommitted_count(), 1);

        let header = thread.commit(2000);
        assert_eq!(thread.uncommitted_count(), 0);
        assert_eq!(header.version, 1);
        assert_ne!(header.prev_header_hash, [0u8; 32]);
    }

    #[test]
    fn test_commit_chains_headers() {
        let kp = Keypair::generate();
        let mut thread = Thread::new(kp, 1000);

        let first_header = thread.commit(1000);
        let first_hash = compute_header_hash(&first_header);

        // Apply a knot
        let dummy_knot = crate::knot::KnotBuilder::transfer(1001)
            .with_payload(norn_types::knot::KnotPayload::Transfer(
                norn_types::knot::TransferPayload {
                    token_id: NATIVE_TOKEN_ID,
                    amount: 100,
                    from: *thread.address(),
                    to: [2u8; 20],
                    memo: None,
                },
            ))
            .build()
            .unwrap();
        thread.apply_knot(dummy_knot, ThreadState::new()).unwrap();

        let second_header = thread.commit(2000);
        assert_eq!(second_header.prev_header_hash, first_hash);
    }
}
