use std::collections::HashMap;

use norn_types::fraud::FraudProofSubmission;
use norn_types::loom::LoomRegistration;
use norn_types::primitives::ThreadId;
use norn_types::weave::{
    BlockTransfer, CommitmentUpdate, LoomAnchor, NameRecordUpdate, NameRegistration, NameTransfer,
    Registration, StakeOperation, TokenBurn, TokenDefinition, TokenMint,
};

use crate::error::WeaveError;

/// Contents drained from the mempool for inclusion in a block.
#[derive(Debug, Clone, Default)]
pub struct BlockContents {
    pub commitments: Vec<CommitmentUpdate>,
    pub registrations: Vec<Registration>,
    pub anchors: Vec<LoomAnchor>,
    pub name_registrations: Vec<NameRegistration>,
    pub name_transfers: Vec<NameTransfer>,
    pub name_record_updates: Vec<NameRecordUpdate>,
    pub fraud_proofs: Vec<FraudProofSubmission>,
    pub transfers: Vec<BlockTransfer>,
    pub token_definitions: Vec<TokenDefinition>,
    pub token_mints: Vec<TokenMint>,
    pub token_burns: Vec<TokenBurn>,
    pub loom_deploys: Vec<LoomRegistration>,
    pub stake_operations: Vec<StakeOperation>,
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
    /// Pending name transfers.
    name_transfers: Vec<NameTransfer>,
    /// Pending name record updates.
    name_record_updates: Vec<NameRecordUpdate>,
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
    /// Pending stake operations.
    stake_operations: Vec<StakeOperation>,
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
            name_transfers: Vec::new(),
            name_record_updates: Vec::new(),
            fraud_proofs: Vec::new(),
            transfers: Vec::new(),
            token_definitions: Vec::new(),
            token_mints: Vec::new(),
            token_burns: Vec::new(),
            loom_deploys: Vec::new(),
            stake_operations: Vec::new(),
            max_size,
        }
    }

    /// Total number of items in the mempool.
    pub fn total_size(&self) -> usize {
        self.commitments.len()
            + self.registrations.len()
            + self.anchors.len()
            + self.name_registrations.len()
            + self.name_transfers.len()
            + self.name_record_updates.len()
            + self.fraud_proofs.len()
            + self.transfers.len()
            + self.token_definitions.len()
            + self.token_mints.len()
            + self.token_burns.len()
            + self.loom_deploys.len()
            + self.stake_operations.len()
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

    /// Add a registration (deduplicated by thread_id).
    pub fn add_registration(&mut self, r: Registration) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .registrations
            .iter()
            .any(|existing| existing.thread_id == r.thread_id)
        {
            return Ok(());
        }
        self.registrations.push(r);
        Ok(())
    }

    /// Add a loom anchor (deduplicated by signature).
    pub fn add_anchor(&mut self, a: LoomAnchor) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .anchors
            .iter()
            .any(|existing| existing.signature == a.signature)
        {
            return Ok(());
        }
        self.anchors.push(a);
        Ok(())
    }

    /// Add a name registration (deduplicated by name — first writer wins).
    pub fn add_name_registration(&mut self, nr: NameRegistration) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .name_registrations
            .iter()
            .any(|existing| existing.name == nr.name)
        {
            return Ok(());
        }
        self.name_registrations.push(nr);
        Ok(())
    }

    /// Add a name transfer (deduplicated by signature).
    pub fn add_name_transfer(&mut self, nt: NameTransfer) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .name_transfers
            .iter()
            .any(|existing| existing.signature == nt.signature)
        {
            return Ok(());
        }
        self.name_transfers.push(nt);
        Ok(())
    }

    /// Add a name record update (deduplicated by signature).
    pub fn add_name_record_update(&mut self, nru: NameRecordUpdate) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .name_record_updates
            .iter()
            .any(|existing| existing.signature == nru.signature)
        {
            return Ok(());
        }
        self.name_record_updates.push(nru);
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

    /// Add a transfer for block inclusion (deduplicated by knot_id).
    pub fn add_transfer(&mut self, t: BlockTransfer) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .transfers
            .iter()
            .any(|existing| existing.knot_id == t.knot_id)
        {
            return Ok(());
        }
        self.transfers.push(t);
        Ok(())
    }

    /// Add a token definition for block inclusion (deduplicated by signature).
    pub fn add_token_definition(&mut self, td: TokenDefinition) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .token_definitions
            .iter()
            .any(|existing| existing.signature == td.signature)
        {
            return Ok(());
        }
        self.token_definitions.push(td);
        Ok(())
    }

    /// Add a token mint for block inclusion (deduplicated by signature).
    pub fn add_token_mint(&mut self, tm: TokenMint) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .token_mints
            .iter()
            .any(|existing| existing.signature == tm.signature)
        {
            return Ok(());
        }
        self.token_mints.push(tm);
        Ok(())
    }

    /// Add a token burn for block inclusion (deduplicated by signature).
    pub fn add_token_burn(&mut self, tb: TokenBurn) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .token_burns
            .iter()
            .any(|existing| existing.signature == tb.signature)
        {
            return Ok(());
        }
        self.token_burns.push(tb);
        Ok(())
    }

    /// Add a stake operation for block inclusion (deduplicated by signature).
    pub fn add_stake_operation(&mut self, op: StakeOperation) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        let op_sig = match &op {
            StakeOperation::Stake { signature, .. } | StakeOperation::Unstake { signature, .. } => {
                *signature
            }
        };
        if self.stake_operations.iter().any(|existing| {
            let existing_sig = match existing {
                StakeOperation::Stake { signature, .. }
                | StakeOperation::Unstake { signature, .. } => signature,
            };
            *existing_sig == op_sig
        }) {
            return Ok(());
        }
        self.stake_operations.push(op);
        Ok(())
    }

    /// Add a loom deployment for block inclusion (deduplicated by signature).
    pub fn add_loom_deploy(&mut self, ld: LoomRegistration) -> Result<(), WeaveError> {
        if self.total_size() >= self.max_size {
            return Err(WeaveError::MempoolFull);
        }
        if self
            .loom_deploys
            .iter()
            .any(|existing| existing.signature == ld.signature)
        {
            return Ok(());
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
        let name_transfers = std::mem::take(&mut self.name_transfers);
        let name_record_updates = std::mem::take(&mut self.name_record_updates);
        let fraud_proofs = std::mem::take(&mut self.fraud_proofs);
        let transfers = std::mem::take(&mut self.transfers);
        let token_definitions = std::mem::take(&mut self.token_definitions);
        let token_mints = std::mem::take(&mut self.token_mints);
        let token_burns = std::mem::take(&mut self.token_burns);
        let loom_deploys = std::mem::take(&mut self.loom_deploys);
        let stake_operations = std::mem::take(&mut self.stake_operations);

        BlockContents {
            commitments,
            registrations,
            anchors,
            name_registrations,
            name_transfers,
            name_record_updates,
            fraud_proofs,
            transfers,
            token_definitions,
            token_mints,
            token_burns,
            loom_deploys,
            stake_operations,
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

    #[test]
    fn test_name_registration_dedup() {
        let mut pool = Mempool::new(100);
        let nr = NameRegistration {
            name: "alice".to_string(),
            owner: [1u8; 20],
            owner_pubkey: [0u8; 32],
            fee_paid: 1_000_000,
            timestamp: 1000,
            signature: [0u8; 64],
        };
        pool.add_name_registration(nr.clone()).unwrap();
        // Adding the same name again should silently succeed but not duplicate.
        pool.add_name_registration(nr).unwrap();

        let contents = pool.drain_for_block(10);
        assert_eq!(contents.name_registrations.len(), 1);
    }
}
