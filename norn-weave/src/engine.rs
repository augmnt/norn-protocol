use std::collections::{HashMap, HashSet};

use norn_crypto::keys::Keypair;
use norn_crypto::merkle::SparseMerkleTree;
use norn_types::constants::MAX_COMMITMENTS_PER_BLOCK;
use norn_types::loom::LoomRegistration;
use norn_types::network::NornMessage;
use norn_types::primitives::*;
use norn_types::weave::{
    BlockTransfer, CommitmentUpdate, NameRecordUpdate, NameRegistration, NameTransfer,
    Registration, StakeOperation, TokenBurn, TokenDefinition, TokenMint, ValidatorSet, WeaveBlock,
    WeaveState,
};
use rayon::prelude::*;

use crate::block;
use crate::commitment;
use crate::consensus::{ConsensusAction, HotStuffEngine};
use crate::mempool::Mempool;
use crate::registration;
use crate::staking::StakingState;

/// The top-level weave engine that orchestrates consensus, mempool, staking, and state.
pub struct WeaveEngine {
    consensus: HotStuffEngine,
    mempool: Mempool,
    staking: StakingState,
    weave_state: WeaveState,
    merkle_tree: SparseMerkleTree,
    keypair: Keypair,
    /// Known thread IDs for duplicate detection.
    known_threads: HashSet<[u8; 20]>,
    /// Known names for duplicate detection.
    known_names: HashSet<String>,
    /// Known name owners for transfer and record update validation.
    known_name_owners: HashMap<String, Address>,
    /// Known tokens for duplicate detection and validation.
    known_tokens: HashMap<TokenId, crate::token::TokenMeta>,
    /// Known token symbols for uniqueness enforcement.
    known_symbols: HashSet<String>,
    /// Known loom IDs for duplicate detection.
    known_looms: HashSet<LoomId>,
    /// Pending validator rewards to be distributed by the node.
    pending_rewards: Option<Vec<(Address, Amount)>>,
    /// Last committed block (for RPC queries).
    last_block: Option<WeaveBlock>,
    /// Current timestamp, set by the node before each tick.
    current_timestamp: Timestamp,
    /// Blocks proposed but not yet committed (multi-validator consensus path).
    pending_blocks: HashMap<Hash, WeaveBlock>,
    /// Height of last finalized (CommitBlock'd) block.
    last_finalized_height: u64,
    /// Total number of blocks finalized through consensus.
    finalized_block_count: u64,
}

impl WeaveEngine {
    /// Create a new weave engine.
    pub fn new(keypair: Keypair, validator_set: ValidatorSet, initial_state: WeaveState) -> Self {
        let staking = StakingState::new(1000, 100);
        let consensus_keypair = Keypair::from_seed(&keypair_seed(&keypair));
        let consensus = HotStuffEngine::new(consensus_keypair, validator_set);
        let mempool = Mempool::new(100_000);
        let merkle_tree = SparseMerkleTree::new();

        Self {
            consensus,
            mempool,
            staking,
            weave_state: initial_state,
            merkle_tree,
            keypair,
            known_threads: HashSet::new(),
            known_names: HashSet::new(),
            known_name_owners: HashMap::new(),
            known_tokens: HashMap::new(),
            known_symbols: HashSet::new(),
            known_looms: HashSet::new(),
            pending_rewards: None,
            last_block: None,
            current_timestamp: 0,
            pending_blocks: HashMap::new(),
            last_finalized_height: 0,
            finalized_block_count: 0,
        }
    }

    /// Handle an incoming network message.
    pub fn on_network_message(&mut self, msg: NornMessage) -> Vec<NornMessage> {
        match msg {
            NornMessage::Commitment(c) => {
                // Validate and add to mempool using real current timestamp.
                if commitment::validate_commitment(&c, None, self.current_timestamp).is_ok() {
                    let _ = self.mempool.add_commitment(c);
                }
                vec![]
            }

            NornMessage::Registration(r) => {
                // Validate and add to mempool.
                if registration::validate_registration(&r, &self.known_threads).is_ok() {
                    let _ = self.mempool.add_registration(r);
                }
                vec![]
            }

            NornMessage::NameRegistration(nr) => {
                if crate::name::validate_name_registration(&nr, &self.known_names).is_ok() {
                    let _ = self.mempool.add_name_registration(nr);
                }
                vec![]
            }

            NornMessage::NameTransfer(nt) => {
                if crate::name::validate_name_transfer(&nt, &self.known_name_owners).is_ok() {
                    let _ = self.mempool.add_name_transfer(nt);
                }
                vec![]
            }

            NornMessage::NameRecordUpdate(nru) => {
                if crate::name::validate_name_record_update(&nru, &self.known_name_owners).is_ok() {
                    let _ = self.mempool.add_name_record_update(nru);
                }
                vec![]
            }

            NornMessage::FraudProof(fp) => {
                if crate::fraud::validate_fraud_proof(&fp).is_ok() {
                    let _ = self.mempool.add_fraud_proof(*fp);
                }
                vec![]
            }

            NornMessage::TokenDefinition(td) => {
                if crate::token::validate_token_definition(
                    &td,
                    &self.known_tokens,
                    &self.known_symbols,
                )
                .is_ok()
                {
                    let _ = self.mempool.add_token_definition(td);
                }
                vec![]
            }

            NornMessage::TokenMint(tm) => {
                if crate::token::validate_token_mint(&tm, &self.known_tokens).is_ok() {
                    let _ = self.mempool.add_token_mint(tm);
                }
                vec![]
            }

            NornMessage::TokenBurn(tb) => {
                if crate::token::validate_token_burn(&tb, &self.known_tokens).is_ok() {
                    let _ = self.mempool.add_token_burn(tb);
                }
                vec![]
            }

            NornMessage::LoomDeploy(ld) => {
                if crate::loom::validate_loom_registration(&ld, &self.known_looms).is_ok() {
                    let _ = self.mempool.add_loom_deploy(*ld);
                }
                vec![]
            }

            NornMessage::StakeOperation(op) => {
                if crate::staking::validate_stake_operation(&op, &self.staking).is_ok() {
                    let _ = self.mempool.add_stake_operation(op);
                }
                vec![]
            }

            NornMessage::Consensus(consensus_msg) => {
                // Extract the sender from the consensus message.
                let from = match extract_sender(&consensus_msg, self.consensus.leader_rotation()) {
                    Some(key) => key,
                    None => return vec![], // Cannot determine sender (empty validator set)
                };
                let actions = self.consensus.on_message(from, consensus_msg);
                self.process_actions(actions)
            }

            NornMessage::Block(weave_block) => {
                // Validate block height is strictly sequential.
                let expected_height = self.weave_state.height + 1;
                if weave_block.height != expected_height && self.weave_state.height > 0 {
                    tracing::debug!(
                        received = weave_block.height,
                        expected = expected_height,
                        "rejecting peer block: non-sequential height"
                    );
                    return vec![];
                }
                // Validate prev_hash continuity (skip for the first block).
                if self.weave_state.height > 0
                    && weave_block.prev_hash != self.weave_state.latest_hash
                {
                    tracing::debug!(
                        received_prev = %hex::encode(weave_block.prev_hash),
                        expected_prev = %hex::encode(self.weave_state.latest_hash),
                        "rejecting peer block: prev_hash mismatch"
                    );
                    return vec![];
                }

                // Validate the block structure.
                let vs = self.staking.active_validators();
                if block::verify_block(&weave_block, &vs).is_err() {
                    return vec![];
                }

                // Reject entire block if ANY commitment is invalid.
                let current_ts = self.current_timestamp;
                let all_commitments_valid = weave_block
                    .commitments
                    .par_iter()
                    .all(|c| commitment::validate_commitment(c, None, current_ts).is_ok());

                if !all_commitments_valid {
                    return vec![];
                }

                // Reject entire block if ANY registration is invalid (including duplicates).
                for r in &weave_block.registrations {
                    if registration::validate_registration(r, &self.known_threads).is_err() {
                        return vec![];
                    }
                }

                // Reject entire block if ANY name registration is invalid or duplicated.
                {
                    let mut seen_names: HashSet<String> = HashSet::new();
                    for nr in &weave_block.name_registrations {
                        if !seen_names.insert(nr.name.clone()) {
                            return vec![];
                        }
                        if crate::name::validate_name_registration(nr, &self.known_names).is_err() {
                            return vec![];
                        }
                    }
                }

                // Reject block if any name transfer is invalid.
                for nt in &weave_block.name_transfers {
                    if crate::name::validate_name_transfer(nt, &self.known_name_owners).is_err() {
                        return vec![];
                    }
                }

                // Reject block if any name record update is invalid.
                for nru in &weave_block.name_record_updates {
                    if crate::name::validate_name_record_update(nru, &self.known_name_owners)
                        .is_err()
                    {
                        return vec![];
                    }
                }

                // Reject block if any token definition is invalid or duplicated within block.
                {
                    let mut seen_token_ids: HashSet<TokenId> = HashSet::new();
                    let mut seen_symbols: HashSet<String> = HashSet::new();
                    for td in &weave_block.token_definitions {
                        if crate::token::validate_token_definition(
                            td,
                            &self.known_tokens,
                            &self.known_symbols,
                        )
                        .is_err()
                        {
                            return vec![];
                        }
                        let token_id = norn_types::token::compute_token_id(
                            &td.creator,
                            &td.name,
                            &td.symbol,
                            td.decimals,
                            td.max_supply,
                            td.timestamp,
                        );
                        if !seen_token_ids.insert(token_id)
                            || !seen_symbols.insert(td.symbol.clone())
                        {
                            return vec![];
                        }
                    }
                }

                // Reject block if any token mint is invalid or intra-block supply exceeded.
                {
                    let mut mint_supply_deltas: HashMap<TokenId, Amount> = HashMap::new();
                    for tm in &weave_block.token_mints {
                        if crate::token::validate_token_mint(tm, &self.known_tokens).is_err() {
                            return vec![];
                        }
                        if let Some(meta) = self.known_tokens.get(&tm.token_id) {
                            if meta.max_supply > 0 {
                                let accumulated =
                                    mint_supply_deltas.entry(tm.token_id).or_insert(0);
                                *accumulated = match accumulated.checked_add(tm.amount) {
                                    Some(v) => v,
                                    None => return vec![],
                                };
                                let projected = match meta.current_supply.checked_add(*accumulated)
                                {
                                    Some(v) => v,
                                    None => return vec![],
                                };
                                if projected > meta.max_supply {
                                    return vec![];
                                }
                            }
                        }
                    }
                }

                // Reject block if any token burn is invalid.
                for tb in &weave_block.token_burns {
                    if crate::token::validate_token_burn(tb, &self.known_tokens).is_err() {
                        return vec![];
                    }
                }

                // Reject block if any loom deploy is invalid or duplicated.
                {
                    let mut seen_loom_ids: HashSet<LoomId> = HashSet::new();
                    for ld in &weave_block.loom_deploys {
                        let loom_id = norn_types::loom::compute_loom_id(ld);
                        if !seen_loom_ids.insert(loom_id) {
                            return vec![];
                        }
                        if crate::loom::validate_loom_registration(ld, &self.known_looms).is_err() {
                            return vec![];
                        }
                    }
                }

                // Reject block if any stake operation is invalid.
                for so in &weave_block.stake_operations {
                    if crate::staking::validate_stake_operation(so, &self.staking).is_err() {
                        return vec![];
                    }
                }

                // All content is valid — apply block state changes.
                self.apply_block_to_state(&weave_block);

                vec![]
            }

            // Other message types are not handled by the weave engine.
            _ => vec![],
        }
    }

    /// Set the current timestamp (called by the node before each tick).
    pub fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.current_timestamp = timestamp;
    }

    /// Handle a periodic tick (multi-validator consensus path).
    pub fn on_tick(&mut self, timestamp: Timestamp) -> Vec<NornMessage> {
        self.current_timestamp = timestamp;
        let mut messages = Vec::new();

        // If we are the leader and have items in the mempool, build and propose a block.
        if self.consensus.is_leader() && !self.mempool.is_empty() {
            let contents = self.mempool.drain_for_block(MAX_COMMITMENTS_PER_BLOCK);
            let weave_block = block::build_block(
                self.weave_state.latest_hash,
                self.weave_state.height,
                contents,
                &self.keypair,
                timestamp,
                [0u8; 32], // state_root provided by node after state application
            );

            let block_hash = weave_block.hash;
            let block_data = borsh::to_vec(&weave_block).unwrap_or_default();

            // Evict stale pending blocks if too many accumulate (consensus stall).
            const MAX_PENDING_BLOCKS: usize = 50;
            if self.pending_blocks.len() >= MAX_PENDING_BLOCKS {
                tracing::warn!(
                    count = self.pending_blocks.len(),
                    "clearing stale pending blocks due to consensus stall"
                );
                self.pending_blocks.clear();
            }

            // Store the block so we can finalize it when CommitBlock arrives.
            self.pending_blocks.insert(block_hash, weave_block);

            let actions = self
                .consensus
                .propose_block(block_hash, block_data, timestamp);
            messages.extend(self.process_actions(actions));
        }

        messages
    }

    /// Convert consensus actions into NornMessages.
    /// Handles CommitBlock by finalizing the pending block and broadcasting it.
    /// Handles RequestViewChange by triggering consensus timeout.
    fn process_actions(&mut self, actions: Vec<ConsensusAction>) -> Vec<NornMessage> {
        let mut messages = Vec::new();

        for action in actions {
            match action {
                ConsensusAction::Broadcast(msg) => {
                    messages.push(NornMessage::Consensus(msg));
                }
                ConsensusAction::SendTo(_to, msg) => {
                    // In a real implementation, this would be addressed.
                    // For now, treat as broadcast.
                    messages.push(NornMessage::Consensus(msg));
                }
                ConsensusAction::CommitBlock(hash) => {
                    // Finalize: apply state changes and broadcast the block.
                    if let Some(block) = self.pending_blocks.remove(&hash) {
                        self.apply_block_to_state(&block);
                        self.last_finalized_height = block.height;
                        self.finalized_block_count += 1;
                        tracing::info!(
                            height = block.height,
                            view = self.consensus.current_view().wrapping_sub(1),
                            finalized = self.finalized_block_count,
                            "block committed via consensus"
                        );
                        messages.push(NornMessage::Block(Box::new(block)));
                    } else {
                        tracing::warn!(
                            hash = hex::encode(hash),
                            "CommitBlock for unknown pending block"
                        );
                    }
                }
                ConsensusAction::RequestViewChange => {
                    // Trigger timeout — collect timeout actions and process them.
                    let timeout_actions = self.consensus.on_timeout();
                    for ta in timeout_actions {
                        match ta {
                            ConsensusAction::Broadcast(msg) => {
                                messages.push(NornMessage::Consensus(msg));
                            }
                            ConsensusAction::SendTo(_, msg) => {
                                messages.push(NornMessage::Consensus(msg));
                            }
                            // Don't recurse into CommitBlock/RequestViewChange from timeout.
                            _ => {}
                        }
                    }
                }
            }
        }

        messages
    }

    /// Produce a block directly, bypassing HotStuff consensus (solo mode).
    /// Drains the mempool, builds a block, applies all state changes, and returns it.
    /// Returns `None` if the mempool is empty.
    pub fn produce_block(&mut self, timestamp: Timestamp, state_root: Hash) -> Option<WeaveBlock> {
        if self.mempool.is_empty() {
            return None;
        }

        let contents = self.mempool.drain_for_block(MAX_COMMITMENTS_PER_BLOCK);
        let weave_block = block::build_block(
            self.weave_state.latest_hash,
            self.weave_state.height,
            contents,
            &self.keypair,
            timestamp,
            state_root,
        );

        self.apply_block_to_state(&weave_block);
        self.last_block = Some(weave_block.clone());
        Some(weave_block)
    }

    /// Apply a block's contents to the engine's internal state.
    /// This is the single source of truth for block application, used by:
    /// - `produce_block()` (solo mode)
    /// - `on_network_message(Block)` (peer block reception)
    /// - `process_actions(CommitBlock)` (multi-validator consensus finalization)
    fn apply_block_to_state(&mut self, block: &WeaveBlock) {
        // Apply commitments.
        for c in &block.commitments {
            let _ = commitment::apply_commitment(&mut self.weave_state, &mut self.merkle_tree, c);
        }
        // Apply registrations.
        for r in &block.registrations {
            let _ =
                registration::apply_registration(&mut self.weave_state, &mut self.merkle_tree, r);
            self.known_threads.insert(r.thread_id);
        }
        // Apply name registrations.
        for nr in &block.name_registrations {
            self.known_names.insert(nr.name.clone());
            self.known_name_owners.insert(nr.name.clone(), nr.owner);
        }
        // Apply name transfers.
        for nt in &block.name_transfers {
            self.known_name_owners.insert(nt.name.clone(), nt.to);
        }
        // Apply token definitions.
        for td in &block.token_definitions {
            let token_id = norn_types::token::compute_token_id(
                &td.creator,
                &td.name,
                &td.symbol,
                td.decimals,
                td.max_supply,
                td.timestamp,
            );
            self.known_symbols.insert(td.symbol.clone());
            self.known_tokens.insert(
                token_id,
                crate::token::TokenMeta {
                    name: td.name.clone(),
                    symbol: td.symbol.clone(),
                    decimals: td.decimals,
                    max_supply: td.max_supply,
                    current_supply: td.initial_supply,
                    creator: td.creator,
                    created_at: td.timestamp,
                },
            );
        }
        // Apply token mints.
        for tm in &block.token_mints {
            if let Some(meta) = self.known_tokens.get_mut(&tm.token_id) {
                meta.current_supply = meta.current_supply.saturating_add(tm.amount);
            }
        }
        // Apply token burns.
        for tb in &block.token_burns {
            if let Some(meta) = self.known_tokens.get_mut(&tb.token_id) {
                match meta.current_supply.checked_sub(tb.amount) {
                    Some(new_supply) => meta.current_supply = new_supply,
                    None => {
                        tracing::warn!(
                            token = %hex::encode(tb.token_id),
                            burn_amount = tb.amount,
                            current_supply = meta.current_supply,
                            "rejecting burn: amount exceeds current supply"
                        );
                    }
                }
            }
        }
        // Apply loom deployments.
        for ld in &block.loom_deploys {
            let loom_id = norn_types::loom::compute_loom_id(ld);
            self.known_looms.insert(loom_id);
        }
        // Apply stake operations to staking state.
        for op in &block.stake_operations {
            match op {
                StakeOperation::Stake { pubkey, amount, .. } => {
                    let addr = norn_crypto::address::pubkey_to_address(pubkey);
                    if let Err(e) = self.staking.stake(*pubkey, addr, *amount) {
                        tracing::debug!("stake operation failed: {}", e);
                    }
                }
                StakeOperation::Unstake { pubkey, amount, .. } => {
                    if let Err(e) = self.staking.unstake(pubkey, *amount, block.height) {
                        tracing::debug!("unstake operation failed: {}", e);
                    }
                }
            }
        }

        // Process epoch (bonding period completions, validator removal).
        let removed = self.staking.process_epoch(block.height);
        if !removed.is_empty() {
            tracing::info!(
                count = removed.len(),
                "validators removed at height {}",
                block.height
            );
        }

        // Update consensus validator set from staking state.
        let new_vs = self.staking.active_validators();
        if !new_vs.is_empty() {
            self.consensus.update_validator_set(new_vs);
        }

        // Update weave state.
        self.weave_state.height = block.height;
        self.weave_state.latest_hash = block.hash;

        // Accumulate fees and update dynamic fee state.
        let commitment_count = block.commitments.len() as u64;
        let total_fee = crate::fees::compute_fee(&self.weave_state.fee_state, commitment_count);
        self.weave_state.fee_state.epoch_fees = self
            .weave_state
            .fee_state
            .epoch_fees
            .saturating_add(total_fee);
        crate::fees::update_fee_state(
            &mut self.weave_state.fee_state,
            commitment_count,
            MAX_COMMITMENTS_PER_BLOCK as u64,
        );

        // Check for epoch boundary — distribute accumulated fees to validators.
        let height = block.height;
        if height > 0
            && height.is_multiple_of(norn_types::constants::BLOCKS_PER_EPOCH)
            && self.weave_state.fee_state.epoch_fees > 0
        {
            let vs = self.staking.active_validators();
            let rewards = crate::fees::compute_reward_distribution(
                &vs,
                self.weave_state.fee_state.epoch_fees,
            );
            if !rewards.is_empty() {
                tracing::info!(
                    height,
                    epoch_fees = self.weave_state.fee_state.epoch_fees,
                    validators = rewards.len(),
                    "distributing epoch rewards to validators"
                );
                self.pending_rewards = Some(rewards);
            }
            self.weave_state.fee_state.epoch_fees = 0;
        }

        self.last_block = Some(block.clone());
    }

    /// Validate and add a commitment update directly to the mempool.
    pub fn add_commitment(
        &mut self,
        c: CommitmentUpdate,
    ) -> Result<bool, crate::error::WeaveError> {
        commitment::validate_commitment(&c, None, self.current_timestamp)?;
        self.mempool.add_commitment(c)?;
        Ok(true)
    }

    /// Validate and add a name registration directly to the mempool.
    pub fn add_name_registration(
        &mut self,
        nr: NameRegistration,
    ) -> Result<bool, crate::error::WeaveError> {
        crate::name::validate_name_registration(&nr, &self.known_names)?;
        self.mempool.add_name_registration(nr)?;
        Ok(true)
    }

    /// Validate and add a name transfer directly to the mempool.
    pub fn add_name_transfer(
        &mut self,
        nt: NameTransfer,
    ) -> Result<bool, crate::error::WeaveError> {
        crate::name::validate_name_transfer(&nt, &self.known_name_owners)?;
        self.mempool.add_name_transfer(nt)?;
        Ok(true)
    }

    /// Validate and add a name record update directly to the mempool.
    pub fn add_name_record_update(
        &mut self,
        nru: NameRecordUpdate,
    ) -> Result<bool, crate::error::WeaveError> {
        crate::name::validate_name_record_update(&nru, &self.known_name_owners)?;
        self.mempool.add_name_record_update(nru)?;
        Ok(true)
    }

    /// Add a verified transfer to the mempool for block inclusion.
    pub fn add_transfer(
        &mut self,
        transfer: BlockTransfer,
    ) -> Result<bool, crate::error::WeaveError> {
        self.mempool.add_transfer(transfer)?;
        Ok(true)
    }

    /// Validate and add a registration directly to the mempool.
    pub fn add_registration(&mut self, r: Registration) -> Result<bool, crate::error::WeaveError> {
        registration::validate_registration(&r, &self.known_threads)?;
        self.mempool.add_registration(r)?;
        Ok(true)
    }

    /// Get the number of registered threads.
    pub fn thread_count(&self) -> u64 {
        self.weave_state.thread_count
    }

    /// Get the set of known thread IDs.
    pub fn known_threads(&self) -> &HashSet<[u8; 20]> {
        &self.known_threads
    }

    /// Get the set of known names.
    pub fn known_names(&self) -> &HashSet<String> {
        &self.known_names
    }

    /// Get the known name owners map.
    pub fn known_name_owners(&self) -> &HashMap<String, Address> {
        &self.known_name_owners
    }

    /// Validate and add a token definition to the mempool.
    pub fn add_token_definition(
        &mut self,
        td: TokenDefinition,
    ) -> Result<TokenId, crate::error::WeaveError> {
        let token_id =
            crate::token::validate_token_definition(&td, &self.known_tokens, &self.known_symbols)?;
        self.mempool.add_token_definition(td)?;
        Ok(token_id)
    }

    /// Validate and add a token mint to the mempool.
    pub fn add_token_mint(&mut self, tm: TokenMint) -> Result<bool, crate::error::WeaveError> {
        crate::token::validate_token_mint(&tm, &self.known_tokens)?;
        self.mempool.add_token_mint(tm)?;
        Ok(true)
    }

    /// Validate and add a token burn to the mempool.
    pub fn add_token_burn(&mut self, tb: TokenBurn) -> Result<bool, crate::error::WeaveError> {
        crate::token::validate_token_burn(&tb, &self.known_tokens)?;
        self.mempool.add_token_burn(tb)?;
        Ok(true)
    }

    /// Get the known tokens map.
    pub fn known_tokens(&self) -> &HashMap<TokenId, crate::token::TokenMeta> {
        &self.known_tokens
    }

    /// Get the known symbols set.
    pub fn known_symbols(&self) -> &HashSet<String> {
        &self.known_symbols
    }

    /// Seed known names, name owners, and threads from persisted state.
    /// Called once at startup so WeaveEngine is in sync with StateManager.
    pub fn seed_known_state(
        &mut self,
        names: impl IntoIterator<Item = String>,
        name_owners: impl IntoIterator<Item = (String, Address)>,
        threads: impl IntoIterator<Item = [u8; 20]>,
    ) {
        self.known_names.extend(names);
        self.known_name_owners.extend(name_owners);
        self.known_threads.extend(threads);
        // Reconcile thread_count with actual known threads after seeding
        self.weave_state.thread_count = self.known_threads.len() as u64;
    }

    /// Validate and add a loom deployment to the mempool.
    pub fn add_loom_deploy(
        &mut self,
        ld: LoomRegistration,
    ) -> Result<LoomId, crate::error::WeaveError> {
        let loom_id = crate::loom::validate_loom_registration(&ld, &self.known_looms)?;
        self.mempool.add_loom_deploy(ld)?;
        Ok(loom_id)
    }

    /// Get the known looms set.
    pub fn known_looms(&self) -> &HashSet<LoomId> {
        &self.known_looms
    }

    /// Seed known looms from persisted state.
    pub fn seed_known_looms(&mut self, looms: impl IntoIterator<Item = LoomId>) {
        self.known_looms.extend(looms);
    }

    /// Seed known tokens from persisted state.
    /// Called once at startup so WeaveEngine is in sync with StateManager.
    pub fn seed_known_tokens(
        &mut self,
        tokens: impl IntoIterator<Item = (TokenId, crate::token::TokenMeta)>,
    ) {
        for (id, meta) in tokens {
            self.known_symbols.insert(meta.symbol.clone());
            self.known_tokens.insert(id, meta);
        }
    }

    /// Get the last committed block.
    pub fn last_block(&self) -> Option<&WeaveBlock> {
        self.last_block.as_ref()
    }

    /// Access the current weave state.
    pub fn weave_state(&self) -> &WeaveState {
        &self.weave_state
    }

    /// Access the mempool.
    pub fn mempool(&self) -> &Mempool {
        &self.mempool
    }

    /// Access the mempool mutably (for RPC handlers that add operations directly).
    pub fn mempool_mut(&mut self) -> &mut Mempool {
        &mut self.mempool
    }

    /// Get the current active validator set.
    pub fn validator_set(&self) -> ValidatorSet {
        self.staking.active_validators()
    }

    /// Take pending validator rewards (if any) after an epoch boundary.
    /// Returns `None` if no rewards are pending.
    pub fn take_pending_rewards(&mut self) -> Option<Vec<(Address, Amount)>> {
        self.pending_rewards.take()
    }

    /// Get the current fee estimate for a single commitment.
    pub fn fee_estimate(&self) -> Amount {
        crate::fees::compute_fee(&self.weave_state.fee_state, 1)
    }

    /// Get a Merkle inclusion proof for a thread.
    pub fn commitment_proof(&self, thread_id: &[u8; 20]) -> norn_crypto::merkle::MerkleProof {
        let key = norn_crypto::hash::blake3_hash(thread_id);
        self.merkle_tree.prove(&key)
    }

    /// Seed staking state from genesis validators.
    pub fn seed_staking(
        &mut self,
        validators: &[norn_types::weave::Validator],
        min_stake: Amount,
        bonding_period: u64,
    ) {
        self.staking = StakingState::new(min_stake, bonding_period);
        for v in validators {
            if let Err(e) = self.staking.stake(v.pubkey, v.address, v.stake) {
                tracing::warn!(
                    validator = hex::encode(v.pubkey),
                    "failed to seed validator stake: {}",
                    e
                );
            }
        }
        // Update consensus with the seeded validator set.
        let new_vs = self.staking.active_validators();
        if !new_vs.is_empty() {
            self.consensus.update_validator_set(new_vs);
        }
    }

    /// Get the height of the last finalized block (via consensus CommitBlock).
    pub fn last_finalized_height(&self) -> u64 {
        self.last_finalized_height
    }

    /// Get the total number of blocks finalized through consensus.
    pub fn finalized_block_count(&self) -> u64 {
        self.finalized_block_count
    }

    /// Handle a consensus timeout (called by the node when no block is committed
    /// within the expected time). Returns messages to broadcast.
    pub fn on_consensus_timeout(&mut self) -> Vec<NornMessage> {
        let actions = self.consensus.on_timeout();
        self.process_actions(actions)
    }

    /// Access the staking state.
    pub fn staking(&self) -> &StakingState {
        &self.staking
    }

    /// Access the staking state mutably (for future slashing support).
    pub fn staking_mut(&mut self) -> &mut StakingState {
        &mut self.staking
    }
}

/// Extract the sender's public key from a consensus message.
///
/// For vote messages, the sender is the voter.
/// For leader-originated messages (Prepare, PreCommit, Commit, NewView),
/// the sender is the leader for the view carried in the message.
fn extract_sender(
    msg: &norn_types::consensus::ConsensusMessage,
    leader_rotation: &crate::leader::LeaderRotation,
) -> Option<PublicKey> {
    use norn_types::consensus::ConsensusMessage;
    match msg {
        ConsensusMessage::PrepareVote(vote) => Some(vote.voter),
        ConsensusMessage::PreCommitVote(vote) => Some(vote.voter),
        ConsensusMessage::CommitVote(vote) => Some(vote.voter),
        ConsensusMessage::ViewChange(tv) => Some(tv.voter),
        ConsensusMessage::Prepare { view, .. } => leader_rotation.leader_for_view(*view).copied(),
        ConsensusMessage::PreCommit { view, .. } => leader_rotation.leader_for_view(*view).copied(),
        ConsensusMessage::Commit { view, .. } => leader_rotation.leader_for_view(*view).copied(),
        ConsensusMessage::NewView { view, .. } => leader_rotation.leader_for_view(*view).copied(),
    }
}

/// Derive a deterministic seed from a keypair for the consensus engine.
/// This allows the consensus engine to have its own Keypair instance while
/// using the same underlying key material.
fn keypair_seed(keypair: &Keypair) -> [u8; 32] {
    keypair.seed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::address::pubkey_to_address;
    use norn_types::weave::{CommitmentUpdate, FeeState, Registration, Validator};

    fn make_weave_state() -> WeaveState {
        WeaveState {
            height: 0,
            latest_hash: [0u8; 32],
            threads_root: [0u8; 32],
            thread_count: 0,
            fee_state: FeeState {
                base_fee: 100,
                fee_multiplier: 1000,
                epoch_fees: 0,
            },
        }
    }

    fn make_validator_set_from_keypair(kp: &Keypair) -> ValidatorSet {
        ValidatorSet {
            validators: vec![Validator {
                pubkey: kp.public_key(),
                address: pubkey_to_address(&kp.public_key()),
                stake: 1000,
                active: true,
            }],
            total_stake: 1000,
            epoch: 0,
        }
    }

    #[test]
    fn test_submit_commitment_to_mempool() {
        let kp = Keypair::generate();
        let vs = make_validator_set_from_keypair(&kp);
        let mut engine = WeaveEngine::new(kp, vs, make_weave_state());

        let commitment = CommitmentUpdate {
            thread_id: [1u8; 20],
            owner: [0u8; 32],
            version: 1,
            state_hash: [1u8; 32],
            prev_commitment_hash: [0u8; 32],
            knot_count: 1,
            timestamp: 1000,
            signature: [0u8; 64],
        };

        // Even with invalid sig, the mempool add happens after validate_commitment
        // which will fail, so mempool stays empty.
        engine.on_network_message(NornMessage::Commitment(commitment));
        // The commitment had an invalid signature, so it should not be in the mempool.
        assert!(engine.mempool().is_empty());
    }

    #[test]
    fn test_submit_registration_to_mempool() {
        let kp = Keypair::generate();
        let vs = make_validator_set_from_keypair(&kp);
        let mut engine = WeaveEngine::new(kp, vs, make_weave_state());

        // Create a properly signed registration.
        let reg_kp = Keypair::generate();
        let thread_id = pubkey_to_address(&reg_kp.public_key());
        let mut reg = Registration {
            thread_id,
            owner: reg_kp.public_key(),
            initial_state_hash: [1u8; 32],
            timestamp: 1000,
            signature: [0u8; 64],
        };
        // Sign it.
        let mut sig_data = Vec::new();
        sig_data.extend_from_slice(&reg.thread_id);
        sig_data.extend_from_slice(&reg.owner);
        sig_data.extend_from_slice(&reg.initial_state_hash);
        sig_data.extend_from_slice(&reg.timestamp.to_le_bytes());
        reg.signature = reg_kp.sign(&sig_data);

        engine.on_network_message(NornMessage::Registration(reg));
        assert!(!engine.mempool().is_empty());
    }

    #[test]
    fn test_engine_creation() {
        let kp = Keypair::generate();
        let vs = make_validator_set_from_keypair(&kp);
        let engine = WeaveEngine::new(kp, vs, make_weave_state());
        assert_eq!(engine.weave_state().height, 0);
    }

    #[test]
    fn test_keypair_seed_preserves_identity() {
        // Bug #3 regression: consensus keypair must match the validator's key.
        let kp = Keypair::generate();
        let seed = keypair_seed(&kp);
        let reconstructed = Keypair::from_seed(&seed);
        assert_eq!(
            kp.public_key(),
            reconstructed.public_key(),
            "consensus keypair must use the same key as the validator"
        );
    }

    #[test]
    fn test_epoch_boundary_triggers_rewards() {
        let kp = Keypair::generate();
        let seed = keypair_seed(&kp);
        let pubkey = kp.public_key();
        let addr = pubkey_to_address(&pubkey);
        let vs = make_validator_set_from_keypair(&kp);
        let mut state = make_weave_state();
        // Set height just before epoch boundary
        state.height = norn_types::constants::BLOCKS_PER_EPOCH - 1;
        state.fee_state.epoch_fees = 5000;
        let mut engine = WeaveEngine::new(kp, vs, state);

        // Seed staking so active_validators() returns something.
        engine.seed_staking(
            &[Validator {
                pubkey,
                address: addr,
                stake: 1000,
                active: true,
            }],
            1000,
            100,
        );

        // Build a minimal block at the epoch boundary height.
        // build_block sets height = prev_height + 1, so pass BLOCKS_PER_EPOCH - 1.
        let block_kp = Keypair::from_seed(&seed);
        let block = crate::block::build_block(
            [0u8; 32],
            norn_types::constants::BLOCKS_PER_EPOCH - 1,
            crate::mempool::BlockContents::default(),
            &block_kp,
            1000,
            [0u8; 32],
        );
        assert_eq!(block.height, norn_types::constants::BLOCKS_PER_EPOCH);
        engine.apply_block_to_state(&block);

        // Rewards should be pending.
        let rewards = engine.take_pending_rewards();
        assert!(rewards.is_some());
        let rewards = rewards.unwrap();
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].1, 5000); // All fees go to single validator
    }

    #[test]
    fn test_epoch_boundary_resets_fees() {
        let kp = Keypair::generate();
        let seed = keypair_seed(&kp);
        let pubkey = kp.public_key();
        let addr = pubkey_to_address(&pubkey);
        let vs = make_validator_set_from_keypair(&kp);
        let mut state = make_weave_state();
        state.height = norn_types::constants::BLOCKS_PER_EPOCH - 1;
        state.fee_state.epoch_fees = 3000;
        let mut engine = WeaveEngine::new(kp, vs, state);

        engine.seed_staking(
            &[Validator {
                pubkey,
                address: addr,
                stake: 1000,
                active: true,
            }],
            1000,
            100,
        );

        let block_kp = Keypair::from_seed(&seed);
        let block = crate::block::build_block(
            [0u8; 32],
            norn_types::constants::BLOCKS_PER_EPOCH - 1,
            crate::mempool::BlockContents::default(),
            &block_kp,
            1000,
            [0u8; 32],
        );
        assert_eq!(block.height, norn_types::constants::BLOCKS_PER_EPOCH);
        engine.apply_block_to_state(&block);

        // Epoch fees should be reset to zero.
        assert_eq!(engine.weave_state().fee_state.epoch_fees, 0);
    }

    #[test]
    fn test_no_rewards_before_epoch_boundary() {
        let kp = Keypair::generate();
        let seed = keypair_seed(&kp);
        let pubkey = kp.public_key();
        let addr = pubkey_to_address(&pubkey);
        let vs = make_validator_set_from_keypair(&kp);
        let mut state = make_weave_state();
        state.height = 499; // Not at epoch boundary (500 != BLOCKS_PER_EPOCH=1000)
        state.fee_state.epoch_fees = 5000;
        let mut engine = WeaveEngine::new(kp, vs, state);

        engine.seed_staking(
            &[Validator {
                pubkey,
                address: addr,
                stake: 1000,
                active: true,
            }],
            1000,
            100,
        );

        let block_kp = Keypair::from_seed(&seed);
        let block = crate::block::build_block(
            [0u8; 32],
            500,
            crate::mempool::BlockContents::default(),
            &block_kp,
            1000,
            [0u8; 32],
        );
        engine.apply_block_to_state(&block);

        // No rewards at non-boundary.
        assert!(engine.take_pending_rewards().is_none());
        // Epoch fees should still be accumulated (not reset).
        assert!(engine.weave_state().fee_state.epoch_fees >= 5000);
    }

    #[test]
    fn test_extract_sender_for_leader_messages() {
        // Bug #4 regression: leader messages must resolve to the leader's key.
        use crate::leader::LeaderRotation;
        use norn_types::consensus::ConsensusMessage;

        let leader_key = [1u8; 32];
        let other_key = [2u8; 32];
        let rotation = LeaderRotation::new(vec![leader_key, other_key]);

        // Prepare message for view 0 -> leader is key[0].
        let msg = ConsensusMessage::Prepare {
            view: 0,
            block_hash: [0u8; 32],
            block_data: vec![],
            justify: None,
        };
        let sender = extract_sender(&msg, &rotation);
        assert_eq!(sender, Some(leader_key));

        // PreCommit for view 1 -> leader is key[1].
        let msg = ConsensusMessage::PreCommit {
            view: 1,
            prepare_qc: norn_types::consensus::QuorumCertificate {
                view: 1,
                block_hash: [0u8; 32],
                phase: norn_types::consensus::ConsensusPhase::Prepare,
                votes: vec![],
            },
        };
        let sender = extract_sender(&msg, &rotation);
        assert_eq!(sender, Some(other_key));
    }
}
