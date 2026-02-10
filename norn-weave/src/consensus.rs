use std::collections::{HashMap, HashSet};

use norn_crypto::hash::blake3_hash;
use norn_crypto::keys::{verify, Keypair};
use norn_types::consensus::*;
use norn_types::primitives::*;
use norn_types::weave::ValidatorSet;

use crate::leader::LeaderRotation;

/// Actions that the HotStuff engine requests the outer layer to perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusAction {
    /// Broadcast a consensus message to all validators.
    Broadcast(ConsensusMessage),
    /// Send a consensus message to a specific validator.
    SendTo(PublicKey, ConsensusMessage),
    /// Commit a block with the given hash.
    CommitBlock(Hash),
    /// Request a view change (timeout occurred).
    RequestViewChange,
}

/// 3-phase HotStuff consensus engine (pure state machine).
pub struct HotStuffEngine {
    keypair: Keypair,
    my_key: PublicKey,
    validator_set: ValidatorSet,
    leader_rotation: LeaderRotation,
    current_view: u64,
    phase: ConsensusPhase,
    /// Collected votes for current view.
    prepare_votes: HashMap<Hash, Vec<Vote>>,
    precommit_votes: HashMap<Hash, Vec<Vote>>,
    commit_votes: HashMap<Hash, Vec<Vote>>,
    /// Timeout tracking.
    timeout_votes: Vec<TimeoutVote>,
    /// QCs.
    prepare_qc: Option<QuorumCertificate>,
    locked_qc: Option<QuorumCertificate>,
    /// Pending block hash for the current view.
    pending_block_hash: Option<Hash>,
}

impl HotStuffEngine {
    /// Create a new HotStuff engine.
    pub fn new(keypair: Keypair, validator_set: ValidatorSet) -> Self {
        let my_key = keypair.public_key();
        let validators: Vec<PublicKey> =
            validator_set.validators.iter().map(|v| v.pubkey).collect();
        let leader_rotation = LeaderRotation::new(validators);

        Self {
            keypair,
            my_key,
            validator_set,
            leader_rotation,
            current_view: 0,
            phase: ConsensusPhase::Prepare,
            prepare_votes: HashMap::new(),
            precommit_votes: HashMap::new(),
            commit_votes: HashMap::new(),
            timeout_votes: Vec::new(),
            prepare_qc: None,
            locked_qc: None,
            pending_block_hash: None,
        }
    }

    /// Update the validator set (e.g., after staking changes).
    pub fn update_validator_set(&mut self, new_vs: ValidatorSet) {
        let validators: Vec<PublicKey> = new_vs.validators.iter().map(|v| v.pubkey).collect();
        self.leader_rotation = LeaderRotation::new(validators);
        self.validator_set = new_vs;
    }

    /// Get the current view number.
    pub fn current_view(&self) -> u64 {
        self.current_view
    }

    /// Get the leader rotation.
    pub fn leader_rotation(&self) -> &LeaderRotation {
        &self.leader_rotation
    }

    /// Check if this node is the leader for the current view.
    pub fn is_leader(&self) -> bool {
        self.leader_rotation
            .is_leader(self.current_view, &self.my_key)
    }

    /// Propose a block (only if we are the leader).
    pub fn propose_block(
        &mut self,
        block_hash: Hash,
        block_data: Vec<u8>,
        _timestamp: Timestamp,
    ) -> Vec<ConsensusAction> {
        if !self.is_leader() {
            return vec![];
        }

        self.pending_block_hash = Some(block_hash);
        self.phase = ConsensusPhase::Prepare;

        let msg = ConsensusMessage::Prepare {
            view: self.current_view,
            block_hash,
            block_data,
            justify: self.prepare_qc.clone(),
        };

        vec![ConsensusAction::Broadcast(msg)]
    }

    /// Handle an incoming consensus message.
    pub fn on_message(&mut self, from: PublicKey, msg: ConsensusMessage) -> Vec<ConsensusAction> {
        // Validate sender is in the validator set.
        if !self.validator_set.contains(&from) {
            return vec![];
        }

        match msg {
            ConsensusMessage::Prepare {
                view,
                block_hash,
                block_data: _,
                justify: _,
            } => self.handle_prepare(from, view, block_hash),

            ConsensusMessage::PrepareVote(vote) => self.handle_prepare_vote(vote),

            ConsensusMessage::PreCommit { view, prepare_qc } => {
                self.handle_precommit(from, view, prepare_qc)
            }

            ConsensusMessage::PreCommitVote(vote) => self.handle_precommit_vote(vote),

            ConsensusMessage::Commit { view, precommit_qc } => {
                self.handle_commit(from, view, precommit_qc)
            }

            ConsensusMessage::CommitVote(vote) => self.handle_commit_vote(vote),

            ConsensusMessage::ViewChange(timeout_vote) => self.handle_view_change(timeout_vote),

            ConsensusMessage::NewView { view, proof } => self.handle_new_view(view, proof),
        }
    }

    /// Handle a timeout event.
    pub fn on_timeout(&mut self) -> Vec<ConsensusAction> {
        let highest_qc_view = self
            .locked_qc
            .as_ref()
            .map(|qc| qc.view)
            .or_else(|| self.prepare_qc.as_ref().map(|qc| qc.view))
            .unwrap_or(0);

        let sig_data = timeout_signing_data(self.current_view, highest_qc_view);
        let signature = self.keypair.sign(&sig_data);

        let tv = TimeoutVote {
            view: self.current_view,
            voter: self.my_key,
            highest_qc_view,
            signature,
        };

        let msg = ConsensusMessage::ViewChange(tv);
        vec![ConsensusAction::Broadcast(msg)]
    }

    /// Advance to the next view and reset per-view state.
    fn advance_view(&mut self) {
        self.current_view += 1;
        self.phase = ConsensusPhase::Prepare;
        self.prepare_votes.clear();
        self.precommit_votes.clear();
        self.commit_votes.clear();
        self.timeout_votes.clear();
        self.pending_block_hash = None;
    }

    // ─── Message Handlers ───────────────────────────────────────────────────

    fn handle_prepare(
        &mut self,
        from: PublicKey,
        view: u64,
        block_hash: Hash,
    ) -> Vec<ConsensusAction> {
        // Only accept Prepare from the leader of this view.
        if !self.leader_rotation.is_leader(view, &from) {
            return vec![];
        }
        if view != self.current_view {
            return vec![];
        }

        self.pending_block_hash = Some(block_hash);

        // Vote PrepareVote.
        let vote = self.make_vote(view, block_hash, ConsensusPhase::Prepare);
        let leader = match self.leader_rotation.leader_for_view(view) {
            Some(l) => *l,
            None => return vec![],
        };

        vec![ConsensusAction::SendTo(
            leader,
            ConsensusMessage::PrepareVote(vote),
        )]
    }

    fn handle_prepare_vote(&mut self, vote: Vote) -> Vec<ConsensusAction> {
        if vote.view != self.current_view {
            return vec![];
        }
        if !self.is_leader() {
            return vec![];
        }

        // Verify vote signature.
        let sig_data = vote_signing_data(vote.view, &vote.block_hash);
        if verify(&sig_data, &vote.signature, &vote.voter).is_err() {
            return vec![];
        }

        let block_hash = vote.block_hash;
        let votes = self.prepare_votes.entry(block_hash).or_default();

        // Avoid duplicate votes from the same voter.
        if votes.iter().any(|v| v.voter == vote.voter) {
            return vec![];
        }
        votes.push(vote);

        if votes.len() >= self.validator_set.quorum_size() {
            // Create prepare QC.
            let qc = QuorumCertificate {
                view: self.current_view,
                block_hash,
                phase: ConsensusPhase::Prepare,
                votes: votes.clone(),
            };
            self.prepare_qc = Some(qc.clone());
            self.phase = ConsensusPhase::PreCommit;

            let msg = ConsensusMessage::PreCommit {
                view: self.current_view,
                prepare_qc: qc,
            };
            return vec![ConsensusAction::Broadcast(msg)];
        }

        vec![]
    }

    fn handle_precommit(
        &mut self,
        from: PublicKey,
        view: u64,
        prepare_qc: QuorumCertificate,
    ) -> Vec<ConsensusAction> {
        if !self.leader_rotation.is_leader(view, &from) {
            return vec![];
        }
        if view != self.current_view {
            return vec![];
        }

        let block_hash = prepare_qc.block_hash;
        self.prepare_qc = Some(prepare_qc);

        // Vote PreCommitVote.
        let vote = self.make_vote(view, block_hash, ConsensusPhase::PreCommit);
        let leader = match self.leader_rotation.leader_for_view(view) {
            Some(l) => *l,
            None => return vec![],
        };

        vec![ConsensusAction::SendTo(
            leader,
            ConsensusMessage::PreCommitVote(vote),
        )]
    }

    fn handle_precommit_vote(&mut self, vote: Vote) -> Vec<ConsensusAction> {
        if vote.view != self.current_view {
            return vec![];
        }
        if !self.is_leader() {
            return vec![];
        }

        let sig_data = vote_signing_data(vote.view, &vote.block_hash);
        if verify(&sig_data, &vote.signature, &vote.voter).is_err() {
            return vec![];
        }

        let block_hash = vote.block_hash;
        let votes = self.precommit_votes.entry(block_hash).or_default();

        if votes.iter().any(|v| v.voter == vote.voter) {
            return vec![];
        }
        votes.push(vote);

        if votes.len() >= self.validator_set.quorum_size() {
            // Create precommit QC (this becomes the locked QC).
            let qc = QuorumCertificate {
                view: self.current_view,
                block_hash,
                phase: ConsensusPhase::PreCommit,
                votes: votes.clone(),
            };
            self.locked_qc = Some(qc.clone());
            self.phase = ConsensusPhase::Commit;

            let msg = ConsensusMessage::Commit {
                view: self.current_view,
                precommit_qc: qc,
            };
            return vec![ConsensusAction::Broadcast(msg)];
        }

        vec![]
    }

    fn handle_commit(
        &mut self,
        from: PublicKey,
        view: u64,
        precommit_qc: QuorumCertificate,
    ) -> Vec<ConsensusAction> {
        if !self.leader_rotation.is_leader(view, &from) {
            return vec![];
        }
        if view != self.current_view {
            return vec![];
        }

        let block_hash = precommit_qc.block_hash;
        self.locked_qc = Some(precommit_qc);

        // Vote CommitVote.
        let vote = self.make_vote(view, block_hash, ConsensusPhase::Commit);
        let leader = match self.leader_rotation.leader_for_view(view) {
            Some(l) => *l,
            None => return vec![],
        };

        vec![ConsensusAction::SendTo(
            leader,
            ConsensusMessage::CommitVote(vote),
        )]
    }

    fn handle_commit_vote(&mut self, vote: Vote) -> Vec<ConsensusAction> {
        if vote.view != self.current_view {
            return vec![];
        }
        if !self.is_leader() {
            return vec![];
        }

        let sig_data = vote_signing_data(vote.view, &vote.block_hash);
        if verify(&sig_data, &vote.signature, &vote.voter).is_err() {
            return vec![];
        }

        let block_hash = vote.block_hash;
        let votes = self.commit_votes.entry(block_hash).or_default();

        if votes.iter().any(|v| v.voter == vote.voter) {
            return vec![];
        }
        votes.push(vote);

        if votes.len() >= self.validator_set.quorum_size() {
            // Commit the block and advance view.
            let action = ConsensusAction::CommitBlock(block_hash);
            self.advance_view();
            return vec![action];
        }

        vec![]
    }

    fn handle_view_change(&mut self, timeout_vote: TimeoutVote) -> Vec<ConsensusAction> {
        // Verify timeout vote signature.
        let sig_data = timeout_signing_data(timeout_vote.view, timeout_vote.highest_qc_view);
        if verify(&sig_data, &timeout_vote.signature, &timeout_vote.voter).is_err() {
            return vec![];
        }

        // Only collect for current view.
        if timeout_vote.view != self.current_view {
            return vec![];
        }

        // Avoid duplicates.
        if self
            .timeout_votes
            .iter()
            .any(|tv| tv.voter == timeout_vote.voter)
        {
            return vec![];
        }

        self.timeout_votes.push(timeout_vote);

        if self.timeout_votes.len() >= self.validator_set.quorum_size() {
            let new_view = self.current_view + 1;

            // Find the highest QC among the timeout votes.
            let highest_qc = self.locked_qc.clone().or_else(|| self.prepare_qc.clone());

            let proof = ViewChangeProof {
                old_view: self.current_view,
                new_view,
                timeout_votes: self.timeout_votes.clone(),
                highest_qc: highest_qc.clone(),
            };

            self.advance_view();

            // If I'm the next leader, broadcast NewView.
            if self.is_leader() {
                let msg = ConsensusMessage::NewView {
                    view: self.current_view,
                    proof,
                };
                return vec![ConsensusAction::Broadcast(msg)];
            }
        }

        vec![]
    }

    fn handle_new_view(&mut self, view: u64, proof: ViewChangeProof) -> Vec<ConsensusAction> {
        // Accept NewView only if it matches the expected new view.
        if view <= self.current_view {
            return vec![];
        }

        // Verify the ViewChangeProof:
        // 1. Must have >= quorum_size timeout votes.
        if proof.timeout_votes.len() < self.validator_set.quorum_size() {
            return vec![];
        }

        // 2. All votes must reference the correct old view.
        if proof.old_view >= proof.new_view {
            return vec![];
        }

        // 3. Verify each timeout vote: valid signature, in validator set, no duplicates.
        let mut seen_voters = HashSet::new();
        for tv in &proof.timeout_votes {
            // Must reference the correct old view.
            if tv.view != proof.old_view {
                return vec![];
            }
            // Must be a known validator.
            if !self.validator_set.contains(&tv.voter) {
                return vec![];
            }
            // No duplicate voters.
            if !seen_voters.insert(tv.voter) {
                return vec![];
            }
            // Verify signature.
            let sig_data = timeout_signing_data(tv.view, tv.highest_qc_view);
            if verify(&sig_data, &tv.signature, &tv.voter).is_err() {
                return vec![];
            }
        }

        // Update state to the new view.
        self.current_view = view;
        self.phase = ConsensusPhase::Prepare;
        self.prepare_votes.clear();
        self.precommit_votes.clear();
        self.commit_votes.clear();
        self.timeout_votes.clear();
        self.pending_block_hash = None;

        // Use the highest QC from the proof.
        if let Some(qc) = proof.highest_qc {
            if qc.phase == ConsensusPhase::PreCommit {
                self.locked_qc = Some(qc.clone());
            }
            self.prepare_qc = Some(qc);
        }

        vec![]
    }

    // ─── Helpers ────────────────────────────────────────────────────────────

    fn make_vote(&self, view: u64, block_hash: Hash, _phase: ConsensusPhase) -> Vote {
        let sig_data = vote_signing_data(view, &block_hash);
        let signature = self.keypair.sign(&sig_data);
        Vote {
            view,
            block_hash,
            voter: self.my_key,
            signature,
        }
    }
}

/// Compute the data to be signed for a vote: blake3(borsh(view, block_hash)).
fn vote_signing_data(view: u64, block_hash: &Hash) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&view.to_le_bytes());
    data.extend_from_slice(block_hash);
    let hash = blake3_hash(&data);
    hash.to_vec()
}

/// Compute the data to be signed for a timeout vote.
fn timeout_signing_data(view: u64, highest_qc_view: u64) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&view.to_le_bytes());
    data.extend_from_slice(&highest_qc_view.to_le_bytes());
    let hash = blake3_hash(&data);
    hash.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_types::weave::Validator;

    fn make_keypairs(n: usize) -> Vec<Keypair> {
        (0..n).map(|i| Keypair::from_seed(&[i as u8; 32])).collect()
    }

    fn make_validator_set(keypairs: &[Keypair]) -> ValidatorSet {
        let validators: Vec<Validator> = keypairs
            .iter()
            .map(|kp| Validator {
                pubkey: kp.public_key(),
                address: [0u8; 20],
                stake: 1000,
                active: true,
            })
            .collect();
        let total_stake = validators.len() as Amount * 1000;
        ValidatorSet {
            validators,
            total_stake,
            epoch: 0,
        }
    }

    #[test]
    fn test_4_validator_full_commit_flow() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);

        // Create engines for all 4 validators.
        let mut engines: Vec<HotStuffEngine> = keypairs
            .into_iter()
            .map(|kp| HotStuffEngine::new(kp, vs.clone()))
            .collect();

        // View 0 => validator 0 is the leader.
        assert!(engines[0].is_leader());
        assert!(!engines[1].is_leader());

        let block_hash = [42u8; 32];
        let block_data = vec![1, 2, 3];

        // Leader proposes.
        let actions = engines[0].propose_block(block_hash, block_data.clone(), 1000);
        assert_eq!(actions.len(), 1);
        let prepare_msg = match &actions[0] {
            ConsensusAction::Broadcast(msg) => msg.clone(),
            _ => panic!("expected broadcast"),
        };

        // Non-leaders receive Prepare, produce PrepareVotes.
        let leader_key = engines[0].my_key;
        let mut prepare_votes = Vec::new();
        for i in 1..4 {
            let from = leader_key;
            let actions = engines[i].on_message(from, prepare_msg.clone());
            assert_eq!(actions.len(), 1);
            match &actions[0] {
                ConsensusAction::SendTo(to, msg) => {
                    assert_eq!(*to, leader_key);
                    prepare_votes.push(msg.clone());
                }
                _ => panic!("expected SendTo"),
            }
        }

        // Leader also votes for its own proposal.
        let own_actions = engines[0].on_message(leader_key, prepare_msg.clone());
        assert_eq!(own_actions.len(), 1);
        match &own_actions[0] {
            ConsensusAction::SendTo(to, msg) => {
                assert_eq!(*to, leader_key);
                prepare_votes.push(msg.clone());
            }
            _ => panic!("expected SendTo"),
        }

        // Feed PrepareVotes to leader. After quorum (3 = 2*1+1), leader broadcasts PreCommit.
        let mut precommit_msg = None;
        for vote_msg in &prepare_votes {
            let voter_key = match vote_msg {
                ConsensusMessage::PrepareVote(v) => v.voter,
                _ => panic!("expected PrepareVote"),
            };
            let actions = engines[0].on_message(voter_key, vote_msg.clone());
            if !actions.is_empty() {
                match &actions[0] {
                    ConsensusAction::Broadcast(msg) => {
                        precommit_msg = Some(msg.clone());
                    }
                    _ => {}
                }
            }
        }
        let precommit_msg = precommit_msg.expect("should have produced PreCommit broadcast");

        // Non-leaders receive PreCommit, vote PreCommitVote.
        let mut precommit_votes = Vec::new();
        for i in 1..4 {
            let actions = engines[i].on_message(leader_key, precommit_msg.clone());
            assert_eq!(actions.len(), 1);
            match &actions[0] {
                ConsensusAction::SendTo(_, msg) => {
                    precommit_votes.push(msg.clone());
                }
                _ => panic!("expected SendTo"),
            }
        }
        // Leader votes too.
        let own_actions = engines[0].on_message(leader_key, precommit_msg.clone());
        assert_eq!(own_actions.len(), 1);
        match &own_actions[0] {
            ConsensusAction::SendTo(_, msg) => {
                precommit_votes.push(msg.clone());
            }
            _ => panic!("expected SendTo"),
        }

        // Feed PreCommitVotes to leader. After quorum, broadcasts Commit.
        let mut commit_msg = None;
        for vote_msg in &precommit_votes {
            let voter_key = match vote_msg {
                ConsensusMessage::PreCommitVote(v) => v.voter,
                _ => panic!("expected PreCommitVote"),
            };
            let actions = engines[0].on_message(voter_key, vote_msg.clone());
            if !actions.is_empty() {
                match &actions[0] {
                    ConsensusAction::Broadcast(msg) => {
                        commit_msg = Some(msg.clone());
                    }
                    _ => {}
                }
            }
        }
        let commit_msg = commit_msg.expect("should have produced Commit broadcast");

        // Non-leaders receive Commit, vote CommitVote.
        let mut commit_votes = Vec::new();
        for i in 1..4 {
            let actions = engines[i].on_message(leader_key, commit_msg.clone());
            assert_eq!(actions.len(), 1);
            match &actions[0] {
                ConsensusAction::SendTo(_, msg) => {
                    commit_votes.push(msg.clone());
                }
                _ => panic!("expected SendTo"),
            }
        }
        // Leader votes.
        let own_actions = engines[0].on_message(leader_key, commit_msg.clone());
        assert_eq!(own_actions.len(), 1);
        match &own_actions[0] {
            ConsensusAction::SendTo(_, msg) => {
                commit_votes.push(msg.clone());
            }
            _ => panic!("expected SendTo"),
        }

        // Feed CommitVotes to leader. After quorum, should produce CommitBlock.
        let mut committed = false;
        for vote_msg in &commit_votes {
            let voter_key = match vote_msg {
                ConsensusMessage::CommitVote(v) => v.voter,
                _ => panic!("expected CommitVote"),
            };
            let actions = engines[0].on_message(voter_key, vote_msg.clone());
            for action in &actions {
                if let ConsensusAction::CommitBlock(hash) = action {
                    assert_eq!(*hash, block_hash);
                    committed = true;
                }
            }
        }
        assert!(committed, "block should have been committed");

        // View should have advanced.
        assert_eq!(engines[0].current_view(), 1);
    }

    #[test]
    fn test_only_leader_can_propose() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);
        let mut engine = HotStuffEngine::new(Keypair::from_seed(&[1u8; 32]), vs);

        // Validator 1 is not the leader for view 0.
        assert!(!engine.is_leader());
        let actions = engine.propose_block([1u8; 32], vec![], 1000);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_rejects_non_validator() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);
        let mut engine = HotStuffEngine::new(Keypair::from_seed(&[0u8; 32]), vs);

        // Unknown validator sends a message.
        let unknown_key = [255u8; 32];
        let msg = ConsensusMessage::Prepare {
            view: 0,
            block_hash: [1u8; 32],
            block_data: vec![],
            justify: None,
        };
        let actions = engine.on_message(unknown_key, msg);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_view_change() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);

        let mut engines: Vec<HotStuffEngine> = keypairs
            .into_iter()
            .map(|kp| HotStuffEngine::new(kp, vs.clone()))
            .collect();

        // All validators timeout.
        let mut timeout_msgs = Vec::new();
        for engine in &mut engines {
            let actions = engine.on_timeout();
            assert_eq!(actions.len(), 1);
            match &actions[0] {
                ConsensusAction::Broadcast(msg) => {
                    timeout_msgs.push((engine.my_key, msg.clone()));
                }
                _ => panic!("expected broadcast"),
            }
        }

        // Feed timeout messages to engine 1 (who will be leader for view 1).
        let mut new_view_broadcast = false;
        for (from, msg) in &timeout_msgs {
            let actions = engines[1].on_message(*from, msg.clone());
            for action in &actions {
                if let ConsensusAction::Broadcast(ConsensusMessage::NewView { view, .. }) = action {
                    assert_eq!(*view, 1);
                    new_view_broadcast = true;
                }
            }
        }
        assert!(
            new_view_broadcast,
            "validator 1 should broadcast NewView for view 1"
        );
        assert_eq!(engines[1].current_view(), 1);
    }

    #[test]
    fn test_new_view_rejects_insufficient_votes() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);
        let mut engine = HotStuffEngine::new(Keypair::from_seed(&[0u8; 32]), vs.clone());

        // Create a proof with only 1 timeout vote (quorum = 3).
        let tv = TimeoutVote {
            view: 0,
            voter: keypairs[0].public_key(),
            highest_qc_view: 0,
            signature: keypairs[0].sign(&timeout_signing_data(0, 0)),
        };
        let proof = ViewChangeProof {
            old_view: 0,
            new_view: 1,
            timeout_votes: vec![tv],
            highest_qc: None,
        };
        let msg = ConsensusMessage::NewView { view: 1, proof };
        let leader_key = keypairs[1].public_key();
        let actions = engine.on_message(leader_key, msg);
        assert!(actions.is_empty());
        assert_eq!(engine.current_view(), 0); // Not advanced.
    }

    #[test]
    fn test_new_view_rejects_invalid_signature() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);
        let mut engine = HotStuffEngine::new(Keypair::from_seed(&[0u8; 32]), vs.clone());

        // Create a proof with 3 timeout votes, one with bad signature.
        let mut votes = Vec::new();
        for kp in &keypairs[0..3] {
            votes.push(TimeoutVote {
                view: 0,
                voter: kp.public_key(),
                highest_qc_view: 0,
                signature: kp.sign(&timeout_signing_data(0, 0)),
            });
        }
        // Corrupt the last signature.
        votes[2].signature[0] ^= 0xff;

        let proof = ViewChangeProof {
            old_view: 0,
            new_view: 1,
            timeout_votes: votes,
            highest_qc: None,
        };
        let msg = ConsensusMessage::NewView { view: 1, proof };
        let leader_key = keypairs[1].public_key();
        let actions = engine.on_message(leader_key, msg);
        assert!(actions.is_empty());
        assert_eq!(engine.current_view(), 0);
    }

    #[test]
    fn test_new_view_rejects_duplicate_voters() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);
        let mut engine = HotStuffEngine::new(Keypair::from_seed(&[0u8; 32]), vs.clone());

        // Create 3 timeout votes where two are from the same voter.
        let tv1 = TimeoutVote {
            view: 0,
            voter: keypairs[0].public_key(),
            highest_qc_view: 0,
            signature: keypairs[0].sign(&timeout_signing_data(0, 0)),
        };
        let tv2 = TimeoutVote {
            view: 0,
            voter: keypairs[1].public_key(),
            highest_qc_view: 0,
            signature: keypairs[1].sign(&timeout_signing_data(0, 0)),
        };
        // Duplicate of tv1.
        let tv3 = tv1.clone();

        let proof = ViewChangeProof {
            old_view: 0,
            new_view: 1,
            timeout_votes: vec![tv1, tv2, tv3],
            highest_qc: None,
        };
        let msg = ConsensusMessage::NewView { view: 1, proof };
        let leader_key = keypairs[1].public_key();
        let actions = engine.on_message(leader_key, msg);
        assert!(actions.is_empty());
        assert_eq!(engine.current_view(), 0);
    }

    #[test]
    fn test_new_view_rejects_wrong_old_view() {
        let keypairs = make_keypairs(4);
        let vs = make_validator_set(&keypairs);
        let mut engine = HotStuffEngine::new(Keypair::from_seed(&[0u8; 32]), vs.clone());

        // Create valid votes but referencing wrong view.
        let mut votes = Vec::new();
        for kp in &keypairs[0..3] {
            votes.push(TimeoutVote {
                view: 5, // Wrong — should match proof.old_view
                voter: kp.public_key(),
                highest_qc_view: 0,
                signature: kp.sign(&timeout_signing_data(5, 0)),
            });
        }

        let proof = ViewChangeProof {
            old_view: 0, // Proof says view 0, but votes say view 5
            new_view: 1,
            timeout_votes: votes,
            highest_qc: None,
        };
        let msg = ConsensusMessage::NewView { view: 1, proof };
        let leader_key = keypairs[1].public_key();
        let actions = engine.on_message(leader_key, msg);
        assert!(actions.is_empty());
        assert_eq!(engine.current_view(), 0);
    }

    #[test]
    fn test_empty_validator_set_no_panic() {
        let vs = ValidatorSet {
            validators: vec![],
            total_stake: 0,
            epoch: 0,
        };
        let kp = Keypair::generate();
        let mut engine = HotStuffEngine::new(kp, vs);
        assert!(!engine.is_leader());

        // Propose should not panic.
        let actions = engine.propose_block([1u8; 32], vec![], 1000);
        assert!(actions.is_empty());

        // Timeout should not panic.
        let actions = engine.on_timeout();
        assert_eq!(actions.len(), 1);
    }
}
