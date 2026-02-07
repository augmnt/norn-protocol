use std::sync::Arc;
use tokio::sync::RwLock;

use norn_crypto::address::pubkey_to_address;
use norn_crypto::keys::Keypair;
use norn_relay::config::RelayConfig;
use norn_relay::relay::{RelayHandle, RelayNode};
use norn_storage::memory::MemoryStore;
use norn_storage::traits::KvStore;
use norn_storage::weave_store::WeaveStore;
use norn_types::constants::BLOCK_TIME_TARGET;
use norn_types::network::NornMessage;
use norn_types::weave::{FeeState, Validator, ValidatorSet, WeaveBlock, WeaveState};
use norn_weave::engine::WeaveEngine;

use crate::config::NodeConfig;
use crate::error::NodeError;
use crate::metrics::NodeMetrics;
use crate::state_manager::StateManager;

/// The main node that ties together all subsystems.
#[allow(dead_code)]
pub struct Node {
    config: NodeConfig,
    weave_engine: Arc<RwLock<WeaveEngine>>,
    state_manager: Arc<RwLock<StateManager>>,
    metrics: Arc<NodeMetrics>,
    rpc_handle: Option<jsonrpsee::server::ServerHandle>,
    block_tx: Option<tokio::sync::broadcast::Sender<crate::rpc::types::BlockInfo>>,
    weave_store: WeaveStore<Arc<dyn KvStore>>,
    relay: Option<RelayNode>,
    relay_rx: Option<tokio::sync::broadcast::Receiver<NornMessage>>,
    relay_handle: Option<RelayHandle>,
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

        // Load genesis state if configured, otherwise use defaults.
        let (validator_set, initial_state) = if let Some(ref genesis_path) = config.genesis_path {
            let (genesis_config, _genesis_block, genesis_state) =
                crate::genesis::load_genesis(genesis_path)?;
            let vs = ValidatorSet {
                validators: genesis_config
                    .validators
                    .iter()
                    .map(|gv| Validator {
                        pubkey: gv.pubkey,
                        address: gv.address,
                        stake: gv.stake,
                        active: true,
                    })
                    .collect(),
                total_stake: genesis_config.validators.iter().map(|v| v.stake).sum(),
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

        let weave_engine = Arc::new(RwLock::new(WeaveEngine::new(
            keypair,
            validator_set,
            effective_state,
        )));

        let metrics = Arc::new(NodeMetrics::new());

        // Rebuild StateManager from persistent storage.
        let ss = crate::state_store::StateStore::new(store.clone());
        let mut sm = match ss.rebuild() {
            Ok(rebuilt) => rebuilt,
            Err(e) => {
                tracing::warn!("Failed to rebuild state from disk, starting fresh: {}", e);
                StateManager::new()
            }
        };
        sm.set_store(ss);
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

        // Start the RPC server if enabled.
        let (rpc_handle, block_tx) = if config.rpc.enabled {
            let (handle, tx) = crate::rpc::server::start_rpc_server(
                &config.rpc.listen_addr,
                weave_engine.clone(),
                state_manager.clone(),
                metrics.clone(),
                relay_handle.clone(),
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
            weave_engine,
            state_manager,
            metrics,
            rpc_handle,
            block_tx,
            weave_store,
            relay,
            relay_rx,
            relay_handle,
        })
    }

    /// Attempt state sync with peers on startup.
    async fn sync_state(&mut self) {
        let handle = match self.relay_handle {
            Some(ref h) => h.clone(),
            None => return,
        };

        tracing::info!("Requesting state sync from peers...");

        let request = NornMessage::StateRequest { current_height: 0 };
        if handle.broadcast(request).await.is_err() {
            tracing::debug!("Failed to send state sync request");
            return;
        }

        // Listen for a StateResponse with a timeout.
        if let Some(ref mut rx) = self.relay_rx {
            let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), async {
                loop {
                    match rx.recv().await {
                        Ok(NornMessage::StateResponse { blocks, tip_height }) => {
                            return Some((blocks, tip_height));
                        }
                        Ok(_) => continue, // ignore other messages during sync
                        Err(_) => return None,
                    }
                }
            })
            .await;

            match timeout {
                Ok(Some((blocks, tip_height))) => {
                    let count = blocks.len();
                    for block in blocks {
                        {
                            let mut sm = self.state_manager.write().await;
                            for reg in &block.registrations {
                                sm.register_thread(reg.thread_id, reg.owner);
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
                    tracing::info!(synced_blocks = count, tip_height, "state sync complete");
                }
                Ok(None) => {
                    tracing::info!("State sync: no response from peers (channel closed)");
                }
                Err(_) => {
                    tracing::info!("State sync: timed out waiting for response (starting fresh)");
                }
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

        tracing::info!("Node is running. Press Ctrl+C to stop.");

        loop {
            // Check for incoming relay messages (non-blocking).
            if let Some(ref mut rx) = self.relay_rx {
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        NornMessage::KnotProposal(knot) => {
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
                                        sm.auto_register_if_needed(transfer.to);
                                        let _ = sm.apply_transfer(
                                            transfer.from,
                                            transfer.to,
                                            transfer.token_id,
                                            transfer.amount,
                                            knot.id,
                                            transfer.memo.clone(),
                                            knot.timestamp,
                                        );
                                    }
                                }
                            }
                        }
                        NornMessage::Block(block) => {
                            // Apply block contents to StateManager.
                            {
                                let mut sm = self.state_manager.write().await;
                                for reg in &block.registrations {
                                    sm.register_thread(reg.thread_id, reg.owner);
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
                        NornMessage::StateRequest { current_height } => {
                            // Respond with blocks the requester is missing.
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
                                    };
                                    tokio::spawn(async move {
                                        let _ = h.broadcast(resp).await;
                                    });
                                }
                            }
                        }
                        NornMessage::StateResponse { blocks, .. } => {
                            // Apply synced blocks.
                            for block in blocks {
                                {
                                    let mut sm = self.state_manager.write().await;
                                    for reg in &block.registrations {
                                        sm.register_thread(reg.thread_id, reg.owner);
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
                        other => {
                            // Forward all other messages to WeaveEngine.
                            let mut engine = self.weave_engine.write().await;
                            engine.set_timestamp(current_timestamp());
                            let _responses = engine.on_network_message(other);
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
                                    };
                                    let _ = tx.send(info);
                                }
                            }
                        } else {
                            let _messages = engine.on_tick(timestamp);
                        }

                        // Update metrics.
                        let state = engine.weave_state();
                        self.metrics.weave_height.set(state.height as i64);
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

    /// Access the weave engine (for testing).
    #[allow(dead_code)]
    pub fn weave_engine(&self) -> &Arc<RwLock<WeaveEngine>> {
        &self.weave_engine
    }

    /// Access the metrics (for testing).
    #[allow(dead_code)]
    pub fn metrics(&self) -> &Arc<NodeMetrics> {
        &self.metrics
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
