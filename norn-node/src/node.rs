use std::sync::Arc;
use tokio::sync::RwLock;

use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::Keypair;
use norn_relay::config::RelayConfig;
use norn_relay::relay::{RelayHandle, RelayNode};
use norn_relay::PeerId;
use norn_spindle::service::SpindleService;
use norn_storage::memory::MemoryStore;
use norn_storage::traits::KvStore;
use norn_storage::weave_store::WeaveStore;
use norn_types::constants::BLOCK_TIME_TARGET;
use norn_types::network::{NetworkId, NornMessage};
use norn_types::weave::{BlockTransfer, FeeState, Validator, ValidatorSet, WeaveBlock, WeaveState};
use norn_weave::engine::WeaveEngine;

use crate::config::NodeConfig;
use crate::error::NodeError;
use crate::metrics::NodeMetrics;
use crate::state_manager::StateManager;

/// The main node that ties together all subsystems.
#[allow(dead_code)] // Fields accessed indirectly via methods, not all read individually
pub struct Node {
    config: NodeConfig,
    genesis_hash: [u8; 32],
    weave_engine: Arc<RwLock<WeaveEngine>>,
    state_manager: Arc<RwLock<StateManager>>,
    metrics: Arc<NodeMetrics>,
    rpc_handle: Option<jsonrpsee::server::ServerHandle>,
    block_tx: Option<tokio::sync::broadcast::Sender<crate::rpc::types::BlockInfo>>,
    weave_store: WeaveStore<Arc<dyn KvStore>>,
    relay: Option<RelayNode>,
    relay_rx: Option<tokio::sync::broadcast::Receiver<(NornMessage, Option<PeerId>)>>,
    relay_handle: Option<RelayHandle>,
    spindle: SpindleService,
}

/// Create a storage backend from the node configuration.
fn create_store(config: &NodeConfig) -> Result<Arc<dyn KvStore>, NodeError> {
    match config.storage.db_type.as_str() {
        "memory" => Ok(Arc::new(MemoryStore::new())),
        "sqlite" => {
            let data_dir = std::path::Path::new(&config.storage.data_dir);
            std::fs::create_dir_all(data_dir)?;
            let db_path = data_dir.join("norn.db");
            let store =
                norn_storage::sqlite::SqliteStore::new(db_path.to_str().unwrap_or("norn.db"))
                    .map_err(NodeError::StorageError)?;
            Ok(Arc::new(store))
        }
        "rocksdb" => {
            let data_dir = std::path::Path::new(&config.storage.data_dir);
            std::fs::create_dir_all(data_dir)?;
            let db_path = data_dir.join("norn.rocksdb");
            let store = norn_storage::rocksdb::RocksDbStore::new(
                db_path.to_str().unwrap_or("norn.rocksdb"),
                None,
            )
            .map_err(NodeError::StorageError)?;
            Ok(Arc::new(store))
        }
        other => Err(NodeError::ConfigError {
            reason: format!(
                "unknown storage backend '{}', expected 'memory', 'sqlite', or 'rocksdb'",
                other
            ),
        }),
    }
}

impl Node {
    /// Create a new node from the given configuration.
    pub async fn new(config: NodeConfig) -> Result<Self, NodeError> {
        // Create or load the validator keypair.
        let keypair = if let Some(ref seed_hex) = config.validator.keypair_seed {
            let seed_bytes = hex::decode(seed_hex).map_err(|e| NodeError::ConfigError {
                reason: format!("invalid keypair seed hex: {}", e),
            })?;
            if seed_bytes.len() != 32 {
                return Err(NodeError::ConfigError {
                    reason: format!("keypair seed must be 32 bytes, got {}", seed_bytes.len()),
                });
            }
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&seed_bytes);
            Keypair::from_seed(&seed)
        } else {
            Keypair::generate()
        };

        // Display the validator address for operators.
        if config.validator.enabled {
            let dim = console::Style::new().dim();
            let green = console::Style::new().green();
            let address = pubkey_to_address(&keypair.public_key());
            println!(
                "  {} 0x{}",
                dim.apply_to("Validator"),
                green.apply_to(hex::encode(address))
            );
            println!();
        }

        // Resolve genesis config: inline > file > defaults.
        let genesis_config_opt: Option<norn_types::genesis::GenesisConfig> =
            if let Some(ref gc) = config.genesis_config {
                Some(gc.clone())
            } else if let Some(ref genesis_path) = config.genesis_path {
                let (gc, _, _) = crate::genesis::load_genesis(genesis_path)?;
                Some(gc)
            } else {
                None
            };

        // Compute genesis hash for chain identity.
        let genesis_hash = if let Some(ref gc) = genesis_config_opt {
            let (genesis_block, _) = crate::genesis::create_genesis_block(gc)?;
            genesis_block.hash
        } else {
            [0u8; 32] // unconfigured chain identity
        };

        if genesis_hash != [0u8; 32] {
            tracing::info!(genesis_hash = %hex::encode(genesis_hash), "chain identity");
        }

        // Build validator set and initial state from genesis config or defaults.
        let (validator_set, initial_state) = if let Some(ref gc) = genesis_config_opt {
            let (_, genesis_state) = crate::genesis::create_genesis_block(gc)?;
            let mut validators: Vec<Validator> = gc
                .validators
                .iter()
                .map(|gv| Validator {
                    pubkey: gv.pubkey,
                    address: gv.address,
                    stake: gv.stake,
                    active: true,
                })
                .collect();
            // Auto-add our keypair as solo validator if genesis has no validators.
            if validators.is_empty() && config.validator.enabled {
                let pubkey = keypair.public_key();
                validators.push(Validator {
                    pubkey,
                    address: pubkey_to_address(&pubkey),
                    stake: 1_000_000_000_000,
                    active: true,
                });
            }
            let total_stake: u128 = validators.iter().map(|v| v.stake).sum();
            let vs = ValidatorSet {
                validators,
                total_stake,
                epoch: 0,
            };
            (vs, genesis_state)
        } else {
            // Default: solo validator set from our own key.
            let validator_set = if config.validator.enabled {
                let pubkey = keypair.public_key();
                ValidatorSet {
                    validators: vec![Validator {
                        pubkey,
                        address: pubkey_to_address(&pubkey),
                        stake: 1_000_000_000_000,
                        active: true,
                    }],
                    total_stake: 1_000_000_000_000,
                    epoch: 0,
                }
            } else {
                ValidatorSet::new(0)
            };
            let initial_state = WeaveState {
                height: 0,
                latest_hash: [0u8; 32],
                threads_root: [0u8; 32],
                thread_count: 0,
                fee_state: FeeState {
                    base_fee: 100,
                    fee_multiplier: 1000,
                    epoch_fees: 0,
                },
            };
            (validator_set, initial_state)
        };

        // Initialize persistent storage.
        let store = create_store(&config)?;
        let weave_store = WeaveStore::new(store.clone());

        // Check schema version before reading any persisted data.
        {
            let ss = crate::state_store::StateStore::new(store.clone());
            ss.check_schema_version()?;
        }

        // Try to load persisted weave state; fall back to genesis/default.
        let effective_state = match weave_store.load_weave_state() {
            Ok(Some(persisted)) => {
                tracing::info!(
                    height = persisted.height,
                    "loaded persisted weave state from disk"
                );
                persisted
            }
            _ => initial_state,
        };

        // Create a spindle keypair from the same seed (before moving keypair into WeaveEngine).
        let spindle_keypair = Keypair::from_seed(&keypair.seed());

        let weave_engine = Arc::new(RwLock::new(WeaveEngine::new(
            keypair,
            validator_set,
            effective_state,
        )));

        let metrics = Arc::new(NodeMetrics::new());

        // Initialize spindle watchtower service.
        let spindle = SpindleService::new(spindle_keypair);

        // Rebuild StateManager from persistent storage.
        let ss = crate::state_store::StateStore::new(store.clone());
        let mut sm = match ss.rebuild() {
            Ok(rebuilt) => rebuilt,
            Err(e) => {
                tracing::warn!("Failed to rebuild state from disk, starting fresh: {}", e);
                StateManager::new()
            }
        };
        // Ensure the schema version is written (stamps fresh stores).
        if let Err(e) = ss.write_schema_version() {
            tracing::warn!("Failed to write schema version: {}", e);
        }
        sm.set_store(ss);

        // Seed WeaveEngine with persisted names and threads from StateManager.
        {
            let names: Vec<String> = sm.registered_names().map(|s| s.to_string()).collect();
            let threads: Vec<[u8; 20]> = sm.registered_thread_ids().copied().collect();
            if !names.is_empty() || !threads.is_empty() {
                tracing::info!(
                    names = names.len(),
                    threads = threads.len(),
                    "seeding WeaveEngine with persisted state"
                );
                let mut engine = weave_engine.write().await;
                engine.seed_known_state(names, threads);
            }
        }

        // Seed WeaveEngine with persisted tokens from StateManager.
        {
            let tokens: Vec<_> = sm
                .registered_tokens()
                .map(|(id, rec)| {
                    (
                        *id,
                        norn_weave::token::TokenMeta {
                            name: rec.name.clone(),
                            symbol: rec.symbol.clone(),
                            decimals: rec.decimals,
                            max_supply: rec.max_supply,
                            current_supply: rec.current_supply,
                            creator: rec.creator,
                            created_at: rec.created_at,
                        },
                    )
                })
                .collect();
            if !tokens.is_empty() {
                tracing::info!(
                    tokens = tokens.len(),
                    "seeding WeaveEngine with persisted tokens"
                );
                let mut engine = weave_engine.write().await;
                engine.seed_known_tokens(tokens);
            }
        }

        let state_manager = Arc::new(RwLock::new(sm));

        // Initialize the relay if networking is configured (before RPC, so handle is available).
        let (relay, relay_rx, relay_handle) =
            if !config.network.boot_nodes.is_empty() || config.network.listen_addr != "0.0.0.0:0" {
                let listen_addr = config
                    .network
                    .listen_addr
                    .parse()
                    .unwrap_or_else(|_| "0.0.0.0:9740".parse().unwrap());
                let boot_nodes = config
                    .network
                    .boot_nodes
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                let relay_config = RelayConfig {
                    listen_addr,
                    boot_nodes,
                    max_connections: config.network.max_connections,
                    keypair_seed: None,
                };
                match RelayNode::new(relay_config).await {
                    Ok(relay_node) => {
                        let rx = relay_node.subscribe();
                        let handle = relay_node.handle();
                        (Some(relay_node), Some(rx), Some(handle))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to initialize relay: {}", e);
                        (None, None, None)
                    }
                }
            } else {
                (None, None, None)
            };

        // Parse network ID.
        let network_id = NetworkId::parse(&config.network_id).unwrap_or(NetworkId::Dev);

        // Process genesis allocations for fresh state.
        {
            let mut sm = state_manager.write().await;
            if sm.latest_block_height() == 0 {
                if let Some(ref gc) = genesis_config_opt {
                    for alloc in &gc.allocations {
                        sm.auto_register_if_needed(alloc.address);
                        if let Err(e) = sm.credit(alloc.address, alloc.token_id, alloc.amount) {
                            tracing::warn!(
                                "Failed to process genesis allocation for {}: {}",
                                hex::encode(alloc.address),
                                e
                            );
                        }
                    }
                    if !gc.allocations.is_empty() {
                        tracing::info!(
                            count = gc.allocations.len(),
                            "processed genesis allocations"
                        );
                    }
                }
            }
        }

        // Register genesis names (idempotent — skips names already registered).
        if let Some(ref gc) = genesis_config_opt {
            if !gc.name_registrations.is_empty() {
                let mut sm = state_manager.write().await;
                let mut registered = 0u32;
                for gnr in &gc.name_registrations {
                    if sm.resolve_name(&gnr.name).is_none() {
                        sm.auto_register_if_needed(gnr.owner);
                        if let Err(e) = sm.apply_peer_name_registration(
                            &gnr.name,
                            gnr.owner,
                            [0u8; 32],
                            gc.timestamp,
                            0,
                        ) {
                            tracing::warn!("failed to register genesis name '{}': {}", gnr.name, e);
                        } else {
                            registered += 1;
                        }
                    }
                }
                if registered > 0 {
                    // Also seed WeaveEngine so it knows about genesis names.
                    let names: Vec<String> = gc
                        .name_registrations
                        .iter()
                        .map(|gnr| gnr.name.clone())
                        .collect();
                    drop(sm);
                    let mut engine = weave_engine.write().await;
                    engine.seed_known_state(names, std::iter::empty());
                    tracing::info!(count = registered, "registered genesis names");
                }
            }
        }

        // Start the RPC server if enabled.
        let (rpc_handle, block_tx) = if config.rpc.enabled {
            let (handle, tx) = crate::rpc::server::start_rpc_server(
                &config.rpc.listen_addr,
                weave_engine.clone(),
                state_manager.clone(),
                metrics.clone(),
                relay_handle.clone(),
                network_id,
                config.validator.enabled,
                config.rpc.api_key.clone(),
            )
            .await?;
            (Some(handle), Some(tx))
        } else {
            (None, None)
        };

        tracing::info!(
            listen = %config.network.listen_addr,
            rpc_enabled = config.rpc.enabled,
            validator_enabled = config.validator.enabled,
            relay_enabled = relay.is_some(),
            "node initialized"
        );

        Ok(Self {
            config,
            genesis_hash,
            weave_engine,
            state_manager,
            metrics,
            rpc_handle,
            block_tx,
            weave_store,
            relay,
            relay_rx,
            relay_handle,
            spindle,
        })
    }

    /// Attempt state sync with peers on startup.
    /// Continues requesting blocks in batches until fully caught up.
    async fn sync_state(&mut self) {
        let handle = match self.relay_handle {
            Some(ref h) => h.clone(),
            None => return,
        };

        tracing::info!("Requesting state sync from peers...");

        let our_genesis_hash = self.genesis_hash;
        let mut current_height: u64 = 0;
        let mut total_synced: u64 = 0;

        loop {
            let request = NornMessage::StateRequest {
                current_height,
                genesis_hash: our_genesis_hash,
            };
            if handle.broadcast(request).await.is_err() {
                tracing::debug!("Failed to send state sync request");
                return;
            }

            // Listen for a StateResponse with a timeout.
            let response = if let Some(ref mut rx) = self.relay_rx {
                let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), async {
                    loop {
                        match rx.recv().await {
                            Ok((
                                NornMessage::StateResponse {
                                    blocks,
                                    tip_height,
                                    genesis_hash,
                                },
                                _source,
                            )) => {
                                if our_genesis_hash != [0u8; 32]
                                    && genesis_hash != [0u8; 32]
                                    && genesis_hash != our_genesis_hash
                                {
                                    tracing::warn!(
                                        ours = %hex::encode(our_genesis_hash),
                                        theirs = %hex::encode(genesis_hash),
                                        "rejecting state sync response: genesis hash mismatch"
                                    );
                                    continue;
                                }
                                return Some((blocks, tip_height));
                            }
                            Ok(_) => continue,
                            Err(_) => return None,
                        }
                    }
                })
                .await;

                match timeout {
                    Ok(Some(resp)) => Some(resp),
                    Ok(None) => {
                        if total_synced == 0 {
                            tracing::info!("State sync: no response from peers (channel closed)");
                        }
                        None
                    }
                    Err(_) => {
                        if total_synced == 0 {
                            tracing::info!(
                                "State sync: timed out waiting for response (starting fresh)"
                            );
                        }
                        None
                    }
                }
            } else {
                None
            };

            match response {
                Some((blocks, tip_height)) => {
                    let batch_size = blocks.len() as u64;
                    if batch_size == 0 {
                        tracing::info!(total_synced, "state sync complete (no more blocks)");
                        break;
                    }

                    let mut max_height = current_height;
                    for block in blocks {
                        if block.height > max_height {
                            max_height = block.height;
                        }
                        {
                            let mut sm = self.state_manager.write().await;
                            for reg in &block.registrations {
                                sm.register_thread(reg.thread_id, reg.owner);
                            }
                            for name_reg in &block.name_registrations {
                                if let Err(e) = sm.apply_peer_name_registration(
                                    &name_reg.name,
                                    name_reg.owner,
                                    name_reg.owner_pubkey,
                                    name_reg.timestamp,
                                    name_reg.fee_paid,
                                ) {
                                    tracing::debug!("skipping known name registration: {}", e);
                                }
                            }
                            // Apply token operations from synced block.
                            for td in &block.token_definitions {
                                if let Err(e) = sm.apply_peer_token_creation(
                                    &td.name,
                                    &td.symbol,
                                    td.decimals,
                                    td.max_supply,
                                    td.initial_supply,
                                    td.creator,
                                    td.creator_pubkey,
                                    td.timestamp,
                                ) {
                                    tracing::debug!("skipping known token definition: {}", e);
                                }
                            }
                            for tm in &block.token_mints {
                                if let Err(e) =
                                    sm.apply_peer_token_mint(tm.token_id, tm.to, tm.amount)
                                {
                                    tracing::debug!("peer token mint failed: {}", e);
                                }
                            }
                            for tb in &block.token_burns {
                                if let Err(e) = sm.apply_peer_token_burn(
                                    tb.token_id,
                                    tb.burner,
                                    tb.burner_pubkey,
                                    tb.amount,
                                ) {
                                    tracing::debug!("peer token burn failed: {}", e);
                                }
                            }
                            for bt in &block.transfers {
                                if !sm.has_transfer(&bt.knot_id) {
                                    sm.auto_register_if_needed(bt.from);
                                    sm.auto_register_if_needed(bt.to);
                                    if let Err(e) = sm.apply_peer_transfer(
                                        bt.from,
                                        bt.to,
                                        bt.token_id,
                                        bt.amount,
                                        bt.knot_id,
                                        bt.memo.clone(),
                                        bt.timestamp,
                                    ) {
                                        tracing::debug!("peer block transfer failed: {}", e);
                                    }
                                }
                            }
                            for commit in &block.commitments {
                                sm.record_commitment(
                                    commit.thread_id,
                                    commit.version,
                                    commit.state_hash,
                                    commit.prev_commitment_hash,
                                    commit.knot_count,
                                );
                            }
                            sm.archive_block(block.clone());
                        }
                        let mut engine = self.weave_engine.write().await;
                        engine.set_timestamp(current_timestamp());
                        let _ = engine.on_network_message(NornMessage::Block(Box::new(block)));
                    }

                    total_synced += batch_size;
                    current_height = max_height;

                    // If we've caught up to the tip, we're done.
                    if current_height >= tip_height {
                        tracing::info!(
                            synced_blocks = total_synced,
                            tip_height,
                            "state sync complete"
                        );
                        break;
                    }

                    tracing::info!(
                        synced_so_far = total_synced,
                        current_height,
                        tip_height,
                        "state sync: requesting next batch..."
                    );
                }
                None => break,
            }
        }
    }

    /// Run the main node event loop.
    pub async fn run(&mut self) -> Result<(), NodeError> {
        let block_interval = tokio::time::interval(BLOCK_TIME_TARGET);
        tokio::pin!(block_interval);

        // Spawn relay run loop in background if available.
        let relay_handle = self.relay.take().map(|mut relay| {
            tokio::spawn(async move {
                if let Err(e) = relay.run().await {
                    tracing::error!("Relay error: {}", e);
                }
            })
        });

        // Attempt state sync with peers before starting the main loop.
        self.sync_state().await;

        // Schedule a sync retry for after mDNS has time to discover peers.
        let needs_sync_retry = {
            let engine = self.weave_engine.read().await;
            engine.weave_state().height == 0
        };
        let mut sync_retry = if needs_sync_retry {
            Some(Box::pin(tokio::time::sleep(
                std::time::Duration::from_secs(15),
            )))
        } else {
            None
        };

        tracing::info!("Node is running. Press Ctrl+C to stop.");

        loop {
            // Check for incoming relay messages (non-blocking).
            if let Some(ref mut rx) = self.relay_rx {
                while let Ok((msg, source_peer)) = rx.try_recv() {
                    match msg {
                        NornMessage::KnotProposal(ref knot) => {
                            // Feed to spindle watchtower for fraud detection.
                            let timestamp = current_timestamp();
                            let fraud_msgs = self.spindle.on_message(&msg, timestamp);
                            for fraud_msg in fraud_msgs {
                                if let Some(ref handle) = self.relay_handle {
                                    let h = handle.clone();
                                    tokio::spawn(async move {
                                        let _ = h.broadcast(fraud_msg).await;
                                    });
                                }
                            }
                            // Validate and apply incoming knot from the network.
                            if let norn_types::knot::KnotPayload::Transfer(ref transfer) =
                                knot.payload
                            {
                                if !knot.signatures.is_empty() && !knot.before_states.is_empty() {
                                    let sender_pubkey = knot.before_states[0].pubkey;
                                    if norn_crypto::keys::verify(
                                        &knot.id,
                                        &knot.signatures[0],
                                        &sender_pubkey,
                                    )
                                    .is_ok()
                                    {
                                        let mut sm = self.state_manager.write().await;
                                        // Dedup: skip if already applied (via RPC or prior gossip).
                                        if sm.has_transfer(&knot.id) {
                                            drop(sm);
                                            continue;
                                        }
                                        sm.auto_register_with_pubkey(transfer.from, sender_pubkey);
                                        sm.auto_register_if_needed(transfer.to);
                                        let applied = sm
                                            .apply_peer_transfer(
                                                transfer.from,
                                                transfer.to,
                                                transfer.token_id,
                                                transfer.amount,
                                                knot.id,
                                                transfer.memo.clone(),
                                                knot.timestamp,
                                            )
                                            .is_ok();
                                        drop(sm);

                                        // Queue for block inclusion so peers can sync.
                                        if applied {
                                            let bt = BlockTransfer {
                                                from: transfer.from,
                                                to: transfer.to,
                                                token_id: transfer.token_id,
                                                amount: transfer.amount,
                                                memo: transfer.memo.clone(),
                                                knot_id: knot.id,
                                                timestamp: knot.timestamp,
                                            };
                                            let mut engine = self.weave_engine.write().await;
                                            let _ = engine.add_transfer(bt);
                                        }
                                    }
                                }
                            }
                        }
                        NornMessage::Block(block) => {
                            // Reject genesis blocks with mismatched hash.
                            if block.height == 0
                                && self.genesis_hash != [0u8; 32]
                                && block.hash != self.genesis_hash
                            {
                                tracing::warn!(
                                    ours = %hex::encode(self.genesis_hash),
                                    theirs = %hex::encode(block.hash),
                                    "rejecting block: genesis hash mismatch"
                                );
                                continue;
                            }
                            // Apply block contents to StateManager.
                            {
                                let mut sm = self.state_manager.write().await;
                                for reg in &block.registrations {
                                    sm.register_thread(reg.thread_id, reg.owner);
                                    self.spindle.watch_thread(reg.thread_id);
                                }
                                for name_reg in &block.name_registrations {
                                    if let Err(e) = sm.apply_peer_name_registration(
                                        &name_reg.name,
                                        name_reg.owner,
                                        name_reg.owner_pubkey,
                                        name_reg.timestamp,
                                        name_reg.fee_paid,
                                    ) {
                                        tracing::debug!("skipping known name registration: {}", e);
                                    }
                                }
                                // Apply token operations from peer block.
                                for td in &block.token_definitions {
                                    if let Err(e) = sm.apply_peer_token_creation(
                                        &td.name,
                                        &td.symbol,
                                        td.decimals,
                                        td.max_supply,
                                        td.initial_supply,
                                        td.creator,
                                        td.creator_pubkey,
                                        td.timestamp,
                                    ) {
                                        tracing::debug!("skipping known token definition: {}", e);
                                    }
                                }
                                for tm in &block.token_mints {
                                    if let Err(e) =
                                        sm.apply_peer_token_mint(tm.token_id, tm.to, tm.amount)
                                    {
                                        tracing::debug!("peer token mint failed: {}", e);
                                    }
                                }
                                for tb in &block.token_burns {
                                    if let Err(e) = sm.apply_peer_token_burn(
                                        tb.token_id,
                                        tb.burner,
                                        tb.burner_pubkey,
                                        tb.amount,
                                    ) {
                                        tracing::debug!("peer token burn failed: {}", e);
                                    }
                                }
                                for bt in &block.transfers {
                                    if !sm.has_transfer(&bt.knot_id) {
                                        sm.auto_register_if_needed(bt.from);
                                        sm.auto_register_if_needed(bt.to);
                                        if let Err(e) = sm.apply_transfer(
                                            bt.from,
                                            bt.to,
                                            bt.token_id,
                                            bt.amount,
                                            bt.knot_id,
                                            bt.memo.clone(),
                                            bt.timestamp,
                                        ) {
                                            tracing::debug!("block transfer failed: {}", e);
                                        }
                                    }
                                }
                                for commit in &block.commitments {
                                    sm.record_commitment(
                                        commit.thread_id,
                                        commit.version,
                                        commit.state_hash,
                                        commit.prev_commitment_hash,
                                        commit.knot_count,
                                    );
                                }
                                sm.archive_block(*block.clone());
                            }
                            // Forward to WeaveEngine.
                            let mut engine = self.weave_engine.write().await;
                            engine.set_timestamp(current_timestamp());
                            let _responses = engine.on_network_message(NornMessage::Block(block));
                        }
                        NornMessage::StateRequest {
                            current_height,
                            genesis_hash,
                        } => {
                            // Reject if genesis hash mismatch.
                            if self.genesis_hash != [0u8; 32]
                                && genesis_hash != [0u8; 32]
                                && genesis_hash != self.genesis_hash
                            {
                                tracing::warn!(
                                    ours = %hex::encode(self.genesis_hash),
                                    theirs = %hex::encode(genesis_hash),
                                    "rejecting state request: genesis hash mismatch"
                                );
                                continue;
                            }
                            // Respond with blocks the requester is missing.
                            // Unicast to the requesting peer if known, otherwise broadcast.
                            if let Some(ref handle) = self.relay_handle {
                                let sm = self.state_manager.read().await;
                                let mut blocks = Vec::new();
                                let tip = sm.latest_block_height();
                                for h in (current_height + 1)..=tip {
                                    if let Some(b) = sm.get_block(h) {
                                        blocks.push(b.clone());
                                        if blocks.len() >= 100 {
                                            break;
                                        }
                                    }
                                }
                                if !blocks.is_empty() {
                                    let h = handle.clone();
                                    let resp = NornMessage::StateResponse {
                                        blocks,
                                        tip_height: tip,
                                        genesis_hash: self.genesis_hash,
                                    };
                                    if let Some(peer_id) = source_peer {
                                        tokio::spawn(async move {
                                            let _ = h.send_to_peer(peer_id, resp).await;
                                        });
                                    } else {
                                        tokio::spawn(async move {
                                            let _ = h.broadcast(resp).await;
                                        });
                                    }
                                }
                            }
                        }
                        NornMessage::StateResponse {
                            blocks,
                            genesis_hash,
                            ..
                        } => {
                            // Reject if genesis hash mismatch.
                            if self.genesis_hash != [0u8; 32]
                                && genesis_hash != [0u8; 32]
                                && genesis_hash != self.genesis_hash
                            {
                                tracing::warn!(
                                    ours = %hex::encode(self.genesis_hash),
                                    theirs = %hex::encode(genesis_hash),
                                    "rejecting state response: genesis hash mismatch"
                                );
                                continue;
                            }
                            // Apply synced blocks.
                            for block in blocks {
                                {
                                    let mut sm = self.state_manager.write().await;
                                    for reg in &block.registrations {
                                        sm.register_thread(reg.thread_id, reg.owner);
                                    }
                                    for name_reg in &block.name_registrations {
                                        if let Err(e) = sm.apply_peer_name_registration(
                                            &name_reg.name,
                                            name_reg.owner,
                                            name_reg.owner_pubkey,
                                            name_reg.timestamp,
                                            name_reg.fee_paid,
                                        ) {
                                            tracing::debug!(
                                                "skipping known name registration: {}",
                                                e
                                            );
                                        }
                                    }
                                    // Apply token operations from state response block.
                                    for td in &block.token_definitions {
                                        if let Err(e) = sm.apply_peer_token_creation(
                                            &td.name,
                                            &td.symbol,
                                            td.decimals,
                                            td.max_supply,
                                            td.initial_supply,
                                            td.creator,
                                            td.creator_pubkey,
                                            td.timestamp,
                                        ) {
                                            tracing::debug!(
                                                "skipping known token definition: {}",
                                                e
                                            );
                                        }
                                    }
                                    for tm in &block.token_mints {
                                        if let Err(e) =
                                            sm.apply_peer_token_mint(tm.token_id, tm.to, tm.amount)
                                        {
                                            tracing::debug!("peer token mint failed: {}", e);
                                        }
                                    }
                                    for tb in &block.token_burns {
                                        if let Err(e) = sm.apply_peer_token_burn(
                                            tb.token_id,
                                            tb.burner,
                                            tb.burner_pubkey,
                                            tb.amount,
                                        ) {
                                            tracing::debug!("peer token burn failed: {}", e);
                                        }
                                    }
                                    for bt in &block.transfers {
                                        if !sm.has_transfer(&bt.knot_id) {
                                            sm.auto_register_if_needed(bt.from);
                                            sm.auto_register_if_needed(bt.to);
                                            if let Err(e) = sm.apply_transfer(
                                                bt.from,
                                                bt.to,
                                                bt.token_id,
                                                bt.amount,
                                                bt.knot_id,
                                                bt.memo.clone(),
                                                bt.timestamp,
                                            ) {
                                                tracing::debug!("block transfer failed: {}", e);
                                            }
                                        }
                                    }
                                    for commit in &block.commitments {
                                        sm.record_commitment(
                                            commit.thread_id,
                                            commit.version,
                                            commit.state_hash,
                                            commit.prev_commitment_hash,
                                            commit.knot_count,
                                        );
                                    }
                                    sm.archive_block(block.clone());
                                }
                                let mut engine = self.weave_engine.write().await;
                                engine.set_timestamp(current_timestamp());
                                let _ =
                                    engine.on_network_message(NornMessage::Block(Box::new(block)));
                            }
                        }
                        NornMessage::UpgradeNotice(notice) => {
                            tracing::warn!(
                                detected_version = notice.protocol_version,
                                message = %notice.message,
                                "upgrade notice: newer protocol version detected on the network"
                            );
                        }
                        other => {
                            // Forward all other messages to WeaveEngine.
                            let mut engine = self.weave_engine.write().await;
                            engine.set_timestamp(current_timestamp());
                            let responses = engine.on_network_message(other);
                            drop(engine);
                            // Route consensus responses through P2P relay.
                            for msg in responses {
                                if let Some(ref handle) = self.relay_handle {
                                    let h = handle.clone();
                                    tokio::spawn(async move {
                                        let _ = h.broadcast(msg).await;
                                    });
                                }
                            }
                        }
                    }
                }
            }

            tokio::select! {
                _ = block_interval.tick() => {
                    if self.config.validator.enabled {
                        let timestamp = current_timestamp();

                        let mut engine = self.weave_engine.write().await;
                        engine.set_timestamp(timestamp);

                        if self.config.validator.solo_mode {
                            // Solo mode: produce blocks directly, bypassing consensus.
                            if let Some(block) = engine.produce_block(timestamp) {
                                tracing::info!(
                                    height = block.height,
                                    commitments = block.commitments.len(),
                                    registrations = block.registrations.len(),
                                    name_registrations = block.name_registrations.len(),
                                    transfers = block.transfers.len(),
                                    "produced block (solo mode)"
                                );
                                self.metrics.blocks_produced.inc();

                                // Persist block and state to storage.
                                self.persist_block(&block, engine.weave_state());

                                // Update StateManager with block contents.
                                {
                                    let mut sm = self.state_manager.write().await;
                                    for reg in &block.registrations {
                                        sm.register_thread(reg.thread_id, reg.owner);
                                        // Watch new threads in spindle for fraud detection.
                                        self.spindle.watch_thread(reg.thread_id);
                                    }
                                    // Apply name registrations (solo — deduct fee locally).
                                    // May fail with "already registered" in multi-validator
                                    // setups where a peer block already applied the name.
                                    for name_reg in &block.name_registrations {
                                        if let Err(e) = sm.register_name(
                                            &name_reg.name,
                                            name_reg.owner,
                                            name_reg.timestamp,
                                        ) {
                                            tracing::debug!("solo name registration skipped: {}", e);
                                        }
                                    }
                                    // Apply token operations (solo — deduct creation fee locally).
                                    for td in &block.token_definitions {
                                        if let Err(e) = sm.create_token(
                                            &td.name, &td.symbol, td.decimals, td.max_supply,
                                            td.initial_supply, td.creator, td.timestamp,
                                        ) {
                                            tracing::debug!("solo token creation skipped: {}", e);
                                        }
                                    }
                                    for tm in &block.token_mints {
                                        if let Err(e) = sm.mint_token(tm.token_id, tm.to, tm.amount) {
                                            tracing::debug!("solo token mint skipped: {}", e);
                                        }
                                    }
                                    for tb in &block.token_burns {
                                        if let Err(e) = sm.burn_token(tb.token_id, tb.burner, tb.amount) {
                                            tracing::debug!("solo token burn skipped: {}", e);
                                        }
                                    }
                                    // Note: transfers are NOT re-applied here — they were
                                    // already applied by the KnotProposal handler above.
                                    // Deduct commitment fees from committers.
                                    let fee_per = norn_weave::fees::compute_fee(
                                        &engine.weave_state().fee_state,
                                        1,
                                    );
                                    for commit in &block.commitments {
                                        sm.record_commitment(
                                            commit.thread_id,
                                            commit.version,
                                            commit.state_hash,
                                            commit.prev_commitment_hash,
                                            commit.knot_count,
                                        );
                                        sm.debit_fee(commit.thread_id, fee_per);
                                    }
                                    sm.archive_block(block.clone());
                                }

                                // Broadcast block to P2P network.
                                if let Some(ref handle) = self.relay_handle {
                                    let h = handle.clone();
                                    let block_msg =
                                        NornMessage::Block(Box::new(block.clone()));
                                    tokio::spawn(async move {
                                        let _ = h.broadcast(block_msg).await;
                                    });
                                }

                                // Notify WebSocket subscribers.
                                if let Some(ref tx) = self.block_tx {
                                    let info = crate::rpc::types::BlockInfo {
                                        height: block.height,
                                        hash: hex::encode(block.hash),
                                        prev_hash: hex::encode(block.prev_hash),
                                        timestamp: block.timestamp,
                                        proposer: hex::encode(block.proposer),
                                        commitment_count: block.commitments.len(),
                                        registration_count: block.registrations.len(),
                                        anchor_count: block.anchors.len(),
                                        fraud_proof_count: block.fraud_proofs.len(),
                                        name_registration_count: block.name_registrations.len(),
                                        transfer_count: block.transfers.len(),
                                        token_definition_count: block.token_definitions.len(),
                                        token_mint_count: block.token_mints.len(),
                                        token_burn_count: block.token_burns.len(),
                                    };
                                    let _ = tx.send(info);
                                }
                            }
                        } else {
                            let messages = engine.on_tick(timestamp);
                            // Route consensus messages through P2P relay.
                            if !messages.is_empty() {
                                if let Some(ref handle) = self.relay_handle {
                                    for msg in messages {
                                        let h = handle.clone();
                                        tokio::spawn(
                                            async move { let _ = h.broadcast(msg).await; },
                                        );
                                    }
                                }
                            }
                        }

                        // Update metrics.
                        let state = engine.weave_state();
                        self.metrics.weave_height.set(state.height as i64);
                        self.metrics
                            .mempool_size
                            .set(engine.mempool().total_size() as i64);
                    }
                }
                _ = async { if let Some(ref mut s) = sync_retry { s.await } else { std::future::pending().await } } => {
                    sync_retry = None;
                    let height = {
                        let engine = self.weave_engine.read().await;
                        engine.weave_state().height
                    };
                    if height == 0 {
                        if let Some(ref handle) = self.relay_handle {
                            tracing::info!("Retrying state sync after peer discovery...");
                            let request = NornMessage::StateRequest {
                                current_height: 0,
                                genesis_hash: self.genesis_hash,
                            };
                            let h = handle.clone();
                            tokio::spawn(async move {
                                let _ = h.broadcast(request).await;
                            });
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received shutdown signal");
                    if let Some(handle) = relay_handle {
                        handle.abort();
                    }
                    self.shutdown().await?;
                    return Ok(());
                }
            }
        }
    }

    /// Gracefully shut down the node.
    pub async fn shutdown(&mut self) -> Result<(), NodeError> {
        tracing::info!("Shutting down node...");

        // Stop the RPC server.
        if let Some(handle) = self.rpc_handle.take() {
            handle.stop().map_err(|e| NodeError::RpcError {
                reason: format!("failed to stop RPC server: {}", e),
            })?;
        }

        tracing::info!("Node shutdown complete");
        Ok(())
    }

    /// Persist a block and the current weave state to storage.
    fn persist_block(&self, block: &WeaveBlock, state: &WeaveState) {
        if let Err(e) = self.weave_store.save_block(block) {
            tracing::warn!("Failed to persist block {}: {}", block.height, e);
        }
        if let Err(e) = self.weave_store.save_weave_state(state) {
            tracing::warn!("Failed to persist weave state: {}", e);
        }
    }
}

/// Get the current UNIX timestamp in seconds.
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NodeConfig;

    fn test_config() -> NodeConfig {
        let mut config = NodeConfig::default();
        // Disable RPC to avoid port conflicts in tests.
        config.rpc.enabled = false;
        config.validator.enabled = false;
        // Clear boot nodes to prevent network connections in tests.
        config.network.boot_nodes.clear();
        config
    }

    #[tokio::test]
    async fn test_node_creation() {
        let config = test_config();
        let node = Node::new(config).await;
        assert!(node.is_ok());
    }

    #[tokio::test]
    async fn test_node_with_seed() {
        let mut config = test_config();
        config.validator.keypair_seed =
            Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string());
        let node = Node::new(config).await;
        assert!(node.is_ok());
    }

    #[tokio::test]
    async fn test_node_with_invalid_seed() {
        let mut config = test_config();
        config.validator.keypair_seed = Some("not-valid-hex".to_string());
        let node = Node::new(config).await;
        assert!(node.is_err());
    }

    #[tokio::test]
    async fn test_node_shutdown() {
        let config = test_config();
        let mut node = Node::new(config).await.unwrap();
        let result = node.shutdown().await;
        assert!(result.is_ok());
    }
}
