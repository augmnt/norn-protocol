use std::collections::HashMap;

use norn_types::fraud::FraudProofSubmission;
use norn_types::loom::LoomRegistration;
use norn_types::primitives::ThreadId;
use norn_types::weave::{
    BlockTransfer, CommitmentUpdate, LoomAnchor, NameRegistration, Registration, TokenBurn,
    TokenDefinition, TokenMint,
};

use crate::error::WeaveError;

/// Contents drained from the mempool for inclusion in a block.
#[derive(Debug, Clone)]
pub struct BlockContents {
    pub commitments: Vec<CommitmentUpdate>,
    pub registrations: Vec<Registration>,
    pub anchors: Vec<LoomAnchor>,
    pub name_registrations: Vec<NameRegistration>,
    pub fraud_proofs: Vec<FraudProofSubmission>,
    pub transfers: Vec<BlockTransfer>,
    pub token_definitions: Vec<TokenDefinition>,
    pub token_mints: Vec<TokenMint>,
    pub token_burns: Vec<TokenBurn>,
    pub loom_deploys: Vec<LoomRegistration>,
}

/// Transaction mempool for pending weave transactions.
pub struct Mempool {
    /// Commitment updates, deduped by thread_id (latest wins).
    commitments: HashMap<ThreadId, CommitmentUpdate>,
    /// Pending registrations.
    registrations: Vec<Registration>,
    /// Pending loom anchors.
    anchors: Vec<LoomAnchor>,
    /// Pending name registrations.
    name_registrations: Vec<NameRegistration>,
    /// Pending fraud proof submissions.
    fraud_proofs: Vec<FraudProofSubmission>,
    /// Pending transfers (from verified knots, for inclusion in blocks).
    transfers: Vec<BlockTransfer>,
    /// Pending token definitions.
    token_definitions: Vec<TokenDefinition>,
    /// Pending token mints.
    token_mints: Vec<TokenMint>,
    /// Pending token burns.
    token_burns: Vec<TokenBurn>,
    /// Pending loom deployments.
    loom_deploys: Vec<LoomRegistration>,
    /// Maximum total number of items in the mempool.
    max_size: usize,
}

impl Mempool {
    /// Create a new mempool with the given capacity.
    pub fn new(max_size: usize) -> Self {
        Self {
            commitments: HashMap::new(),
            registrations: Vec::new(),
            anchors: Vec::new(),
            name_registrations: Vec::new(),
            fraud_proofs: Vec::new(),
            transfers: Vec::new(),
            token_definitions: Vec::new(),
            token_mints: Vec::new(),
            token_burns: Vec::new(),
            loom_deploys: Vec::new(),
            max_size,
        }
    }

    /// Total number of items in the mempool.
    pub fn total_size(&self) -> usize {
        self.commitments.len()
            + self.registrations.len()
            + self.anchors.len()
            + self.name_registrations.len()
            + self.fraud_proofs.len()
            + self.transfers.len()
            + self.token_definitions.len()
            + self.token_mints.len()
            + self.token_burns.len()
            + self.loom_deploys.len()
    }

    /// Add a commitment update (deduplicates by thread_id; latest wins).
    pub fn add_commitment(&mut self, c: CommitmentUpdate) -> Result<(), WeaveError> {
        // Dedup allows replacing, so only check capacity if it's a new thread_id.
        if !self.commitments.contains_key(&c.thread_id) && self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.commitments.insert(c.thread_id, c);
        Ok(())
    }

    /// Add a registration.
    pub fn add_registration(&mut self, r: Registration) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.registrations.push(r);
        Ok(())
    }

    /// Add a loom anchor.
    pub fn add_anchor(&mut self, a: LoomAnchor) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.anchors.push(a);
        Ok(())
    }

    /// Add a name registration.
    pub fn add_name_registration(&mut self, nr: NameRegistration) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.name_registrations.push(nr);
        Ok(())
    }

    /// Add a fraud proof submission.
    pub fn add_fraud_proof(&mut self, fp: FraudProofSubmission) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.fraud_proofs.push(fp);
        Ok(())
    }

    /// Add a transfer for block inclusion.
    pub fn add_transfer(&mut self, t: BlockTransfer) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.transfers.push(t);
        Ok(())
    }

    /// Add a token definition for block inclusion.
    pub fn add_token_definition(&mut self, td: TokenDefinition) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.token_definitions.push(td);
        Ok(())
    }

    /// Add a token mint for block inclusion.
    pub fn add_token_mint(&mut self, tm: TokenMint) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.token_mints.push(tm);
        Ok(())
    }

    /// Add a token burn for block inclusion.
    pub fn add_token_burn(&mut self, tb: TokenBurn) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.token_burns.push(tb);
        Ok(())
    }

    /// Add a loom deployment for block inclusion.
    pub fn add_loom_deploy(&mut self, ld: LoomRegistration) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        self.loom_deploys.push(ld);
        Ok(())
    }

    /// Drain items from the mempool for block building.
    /// Takes up to `max_commitments` commitment updates, and all registrations,
    /// anchors, and fraud proofs.
    pub fn drain_for_block(&mut self, max_commitments: usize) -> BlockContents {
        let commitments: Vec<CommitmentUpdate> = if self.commitments.len() <= max_commitments {
            // Take all — drain is safe here since we want everything.
            self.commitments.drain().map(|(_, v)| v).collect()
        } else {
            // Only remove the first `max_commitments` entries, preserving the rest.
            let keys: Vec<ThreadId> = self
                .commitments
                .keys()
                .take(max_commitments)
                .copied()
                .collect();
            keys.into_iter()
                .filter_map(|k| self.commitments.remove(&k))
                .collect()
        };

        let registrations = std::mem::take(&mut self.registrations);
        let anchors = std::mem::take(&mut self.anchors);
        let name_registrations = std::mem::take(&mut self.name_registrations);
        let fraud_proofs = std::mem::take(&mut self.fraud_proofs);
        let transfers = std::mem::take(&mut self.transfers);
        let token_definitions = std::mem::take(&mut self.token_definitions);
        let token_mints = std::mem::take(&mut self.token_mints);
        let token_burns = std::mem::take(&mut self.token_burns);
        let loom_deploys = std::mem::take(&mut self.loom_deploys);

        BlockContents {
            commitments,
            registrations,
            anchors,
            name_registrations,
            fraud_proofs,
            transfers,
            token_definitions,
            token_mints,
            token_burns,
            loom_deploys,
        }
    }

    /// Number of pending commitment updates.
    pub fn commitment_count(&self) -> usize {
        self.commitments.len()
    }

    /// Whether the mempool has no pending items.
    pub fn is_empty(&self) -> bool {
        self.total_size() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::primitives::*;

    fn make_commitment(thread_id: ThreadId, version: Version) -> CommitmentUpdate {
        CommitmentUpdate {
            thread_id,
            owner: [0u8; 32],
            version,
            state_hash: [0u8; 32],
            prev_commitment_hash: [0u8; 32],
            knot_count: 1,
            timestamp: 1000,
            signature: [0u8; 64],
        }
    }

    fn make_registration(thread_id: ThreadId) -> Registration {
        Registration {
            thread_id,
            owner: [0u8; 32],
            initial_state_hash: [0u8; 32],
            timestamp: 1000,
            signature: [0u8; 64],
        }
    }

    #[test]
    fn test_add_and_drain() {
        let mut pool = Mempool::new(100);
        let c = make_commitment([1u8; 20], 1);
        pool.add_commitment(c).unwrap();

        let r = make_registration([2u8; 20]);
        pool.add_registration(r).unwrap();

        assert_eq!(pool.commitment_count(), 1);
        assert!(!pool.is_empty());

        let contents = pool.drain_for_block(10);
        assert_eq!(contents.commitments.len(), 1);
        assert_eq!(contents.registrations.len(), 1);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_dedup_by_thread_id() {
        let mut pool = Mempool::new(100);
        let tid = [1u8; 20];
        pool.add_commitment(make_commitment(tid, 1)).unwrap();
        pool.add_commitment(make_commitment(tid, 2)).unwrap();

        assert_eq!(pool.commitment_count(), 1);

        let contents = pool.drain_for_block(10);
        assert_eq!(contents.commitments.len(), 1);
        assert_eq!(contents.commitments[0].version, 2);
    }

    #[test]
    fn test_capacity_limits() {
        let mut pool = Mempool::new(2);
        pool.add_commitment(make_commitment([1u8; 20], 1)).unwrap();
        pool.add_registration(make_registration([2u8; 20])).unwrap();

        // Pool is full (2 items).
        let result = pool.add_registration(make_registration([3u8; 20]));
        assert!(result.is_err());
    }

    #[test]
    fn test_dedup_does_not_count_as_new() {
        let mut pool = Mempool::new(2);
        pool.add_commitment(make_commitment([1u8; 20], 1)).unwrap();
        pool.add_registration(make_registration([2u8; 20])).unwrap();

        // Replacing an existing commitment should work even at capacity.
        pool.add_commitment(make_commitment([1u8; 20], 2)).unwrap();
        assert_eq!(pool.commitment_count(), 1);
    }

    #[test]
    fn test_drain_max_commitments() {
        let mut pool = Mempool::new(100);
        for i in 0..10u8 {
            let mut tid = [0u8; 20];
            tid[0] = i;
            pool.add_commitment(make_commitment(tid, 1)).unwrap();
        }

        let contents = pool.drain_for_block(3);
        assert_eq!(contents.commitments.len(), 3);
    }

    #[test]
    fn test_empty_mempool() {
        let pool = Mempool::new(100);
        assert!(pool.is_empty());
        assert_eq!(pool.commitment_count(), 0);
    }

    #[test]
    fn test_drain_preserves_excess_commitments() {
        // Bug #5 regression: drain should only remove up to max_commitments,
        // keeping the rest in the mempool.
        let mut pool = Mempool::new(100);
        for i in 0..10u8 {
            let mut tid = [0u8; 20];
            tid[0] = i;
            pool.add_commitment(make_commitment(tid, 1)).unwrap();
        }
        assert_eq!(pool.commitment_count(), 10);

        let contents = pool.drain_for_block(3);
        assert_eq!(contents.commitments.len(), 3);
        // The remaining 7 commitments must still be in the mempool.
        assert_eq!(pool.commitment_count(), 7);
    }
}
