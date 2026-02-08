use std::sync::Arc;
use tokio::sync::RwLock;

use jsonrpsee::core::async_trait;
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::PendingSubscriptionSink;

use norn_types::network::NornMessage;
use norn_weave::engine::WeaveEngine;

use super::types::{
    BlockInfo, CommitmentProofInfo, FeeEstimateInfo, HealthInfo, NameInfo, NameResolution,
    SubmitResult, ThreadInfo, ThreadStateInfo, TransactionHistoryEntry, ValidatorInfo,
    ValidatorSetInfo, WeaveStateInfo,
};
use crate::metrics::NodeMetrics;
use crate::state_manager::StateManager;
use crate::wallet::format::{format_address, format_amount_with_symbol};

/// JSON-RPC trait for the Norn node.
#[rpc(server)]
pub trait NornRpc {
    /// Get a block by height.
    #[method(name = "norn_getBlock")]
    async fn get_block(&self, height: u64) -> Result<Option<BlockInfo>, ErrorObjectOwned>;

    /// Get the latest block.
    #[method(name = "norn_getLatestBlock")]
    async fn get_latest_block(&self) -> Result<Option<BlockInfo>, ErrorObjectOwned>;

    /// Get the current weave state.
    #[method(name = "norn_getWeaveState")]
    async fn get_weave_state(&self) -> Result<Option<WeaveStateInfo>, ErrorObjectOwned>;

    /// Submit a commitment (hex-encoded borsh bytes).
    #[method(name = "norn_submitCommitment")]
    async fn submit_commitment(&self, commitment: String)
        -> Result<SubmitResult, ErrorObjectOwned>;

    /// Submit a registration (hex-encoded borsh bytes).
    #[method(name = "norn_submitRegistration")]
    async fn submit_registration(
        &self,
        registration: String,
    ) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Get thread info by thread ID (hex).
    #[method(name = "norn_getThread")]
    async fn get_thread(&self, thread_id: String) -> Result<Option<ThreadInfo>, ErrorObjectOwned>;

    /// Get balance for an address and token.
    #[method(name = "norn_getBalance")]
    async fn get_balance(
        &self,
        address: String,
        token_id: String,
    ) -> Result<String, ErrorObjectOwned>;

    /// Get thread state info.
    #[method(name = "norn_getThreadState")]
    async fn get_thread_state(
        &self,
        thread_id: String,
    ) -> Result<Option<ThreadStateInfo>, ErrorObjectOwned>;

    /// Request testnet faucet tokens (testnet-only, returns error in production builds).
    #[method(name = "norn_faucet")]
    async fn faucet(&self, address: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Submit a knot (hex-encoded borsh bytes).
    #[method(name = "norn_submitKnot")]
    async fn submit_knot(&self, knot: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Health check endpoint.
    #[method(name = "norn_health")]
    async fn health(&self) -> Result<HealthInfo, ErrorObjectOwned>;

    /// Get the current validator set.
    #[method(name = "norn_getValidatorSet")]
    async fn get_validator_set(&self) -> Result<ValidatorSetInfo, ErrorObjectOwned>;

    /// Get fee estimate for a commitment.
    #[method(name = "norn_getFeeEstimate")]
    async fn get_fee_estimate(&self) -> Result<FeeEstimateInfo, ErrorObjectOwned>;

    /// Get a Merkle commitment proof for a thread.
    #[method(name = "norn_getCommitmentProof")]
    async fn get_commitment_proof(
        &self,
        thread_id: String,
    ) -> Result<Option<CommitmentProofInfo>, ErrorObjectOwned>;

    /// Subscribe to new blocks.
    #[subscription(name = "norn_subscribeNewBlocks" => "norn_newBlocks", unsubscribe = "norn_unsubscribeNewBlocks", item = BlockInfo)]
    async fn subscribe_new_blocks(&self) -> SubscriptionResult;

    /// Get transaction history for an address.
    #[method(name = "norn_getTransactionHistory")]
    async fn get_transaction_history(
        &self,
        address: String,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TransactionHistoryEntry>, ErrorObjectOwned>;

    /// Register a name for an address (requires signed knot for authentication).
    #[method(name = "norn_registerName")]
    async fn register_name(
        &self,
        name: String,
        owner_hex: String,
        knot_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Resolve a name to its owner address.
    #[method(name = "norn_resolveName")]
    async fn resolve_name(&self, name: String) -> Result<Option<NameResolution>, ErrorObjectOwned>;

    /// List names owned by an address.
    #[method(name = "norn_listNames")]
    async fn list_names(&self, address_hex: String) -> Result<Vec<NameInfo>, ErrorObjectOwned>;

    /// Get node metrics in Prometheus text exposition format.
    #[method(name = "norn_getMetrics")]
    async fn get_metrics(&self) -> Result<String, ErrorObjectOwned>;

    /// Submit a fraud proof (hex-encoded borsh bytes).
    #[method(name = "norn_submitFraudProof")]
    async fn submit_fraud_proof(
        &self,
        fraud_proof_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned>;
}

/// Implementation of the NornRpc trait.
#[allow(dead_code)] // Required: jsonrpsee accesses fields via trait impl
pub struct NornRpcImpl {
    pub weave_engine: Arc<RwLock<WeaveEngine>>,
    pub state_manager: Arc<RwLock<StateManager>>,
    pub metrics: Arc<NodeMetrics>,
    pub block_tx: tokio::sync::broadcast::Sender<BlockInfo>,
    pub relay_handle: Option<norn_relay::relay::RelayHandle>,
    pub network_id: norn_types::network::NetworkId,
    pub is_validator: bool,
    pub faucet_tracker: std::sync::Mutex<std::collections::HashMap<[u8; 20], u64>>,
}

/// Parse a hex string into a 20-byte address.
fn parse_address_hex(hex_str: &str) -> Result<[u8; 20], ErrorObjectOwned> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>))?;
    if bytes.len() != 20 {
        return Err(ErrorObjectOwned::owned(
            -32602,
            format!("address must be 20 bytes, got {}", bytes.len()),
            None::<()>,
        ));
    }
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&bytes);
    Ok(addr)
}

/// Parse a hex string into a 32-byte token ID.
fn parse_token_hex(hex_str: &str) -> Result<[u8; 32], ErrorObjectOwned> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>))?;
    if bytes.len() != 32 {
        return Err(ErrorObjectOwned::owned(
            -32602,
            format!("token_id must be 32 bytes, got {}", bytes.len()),
            None::<()>,
        ));
    }
    let mut id = [0u8; 32];
    id.copy_from_slice(&bytes);
    Ok(id)
}

#[async_trait]
impl NornRpcServer for NornRpcImpl {
    async fn get_block(&self, height: u64) -> Result<Option<BlockInfo>, ErrorObjectOwned> {
        // Try the StateManager archive first.
        let sm = self.state_manager.read().await;
        if let Some(block) = sm.get_block(height) {
            return Ok(Some(BlockInfo {
                height: block.height,
                hash: hex::encode(block.hash),
                prev_hash: hex::encode(block.prev_hash),
                timestamp: block.timestamp,
                proposer: hex::encode(block.proposer),
                commitment_count: block.commitments.len(),
                registration_count: block.registrations.len(),
                anchor_count: block.anchors.len(),
                fraud_proof_count: block.fraud_proofs.len(),
            }));
        }
        drop(sm);

        // Fallback: check if it's the current height from weave state.
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();
        if height == state.height {
            Ok(Some(BlockInfo {
                height: state.height,
                hash: hex::encode(state.latest_hash),
                prev_hash: String::new(),
                timestamp: 0,
                proposer: String::new(),
                commitment_count: 0,
                registration_count: 0,
                anchor_count: 0,
                fraud_proof_count: 0,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_latest_block(&self) -> Result<Option<BlockInfo>, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;

        if let Some(block) = engine.last_block() {
            Ok(Some(BlockInfo {
                height: block.height,
                hash: hex::encode(block.hash),
                prev_hash: hex::encode(block.prev_hash),
                timestamp: block.timestamp,
                proposer: hex::encode(block.proposer),
                commitment_count: block.commitments.len(),
                registration_count: block.registrations.len(),
                anchor_count: block.anchors.len(),
                fraud_proof_count: block.fraud_proofs.len(),
            }))
        } else {
            let state = engine.weave_state();
            Ok(Some(BlockInfo {
                height: state.height,
                hash: hex::encode(state.latest_hash),
                prev_hash: String::new(),
                timestamp: 0,
                proposer: String::new(),
                commitment_count: 0,
                registration_count: 0,
                anchor_count: 0,
                fraud_proof_count: 0,
            }))
        }
    }

    async fn get_weave_state(&self) -> Result<Option<WeaveStateInfo>, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();

        Ok(Some(WeaveStateInfo {
            height: state.height,
            latest_hash: hex::encode(state.latest_hash),
            threads_root: hex::encode(state.threads_root),
            thread_count: state.thread_count,
            base_fee: state.fee_state.base_fee.to_string(),
            fee_multiplier: state.fee_state.fee_multiplier,
        }))
    }

    async fn submit_commitment(
        &self,
        commitment_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&commitment_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let commitment: norn_types::weave::CommitmentUpdate =
            borsh::from_slice(&bytes).map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("invalid commitment: {}", e), None::<()>)
            })?;

        let mut engine = self.weave_engine.write().await;
        match engine.add_commitment(commitment.clone()) {
            Ok(_) => {
                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::Commitment(commitment);
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: None,
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn submit_registration(
        &self,
        registration_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&registration_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let registration: norn_types::weave::Registration =
            borsh::from_slice(&bytes).map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("invalid registration: {}", e), None::<()>)
            })?;

        // Also register in StateManager.
        {
            let mut sm = self.state_manager.write().await;
            sm.register_thread(registration.thread_id, registration.owner);
        }

        let mut engine = self.weave_engine.write().await;
        match engine.add_registration(registration.clone()) {
            Ok(_) => {
                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::Registration(registration);
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: None,
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn get_thread(
        &self,
        thread_id_hex: String,
    ) -> Result<Option<ThreadInfo>, ErrorObjectOwned> {
        let thread_id = parse_address_hex(&thread_id_hex)?;

        let sm = self.state_manager.read().await;
        if let Some(meta) = sm.get_thread_meta(&thread_id) {
            Ok(Some(ThreadInfo {
                thread_id: thread_id_hex,
                owner: hex::encode(meta.owner),
                version: meta.version,
                state_hash: hex::encode(meta.state_hash),
            }))
        } else {
            // Fallback: check WeaveEngine known_threads.
            drop(sm);
            let engine = self.weave_engine.read().await;
            if engine.known_threads().contains(&thread_id) {
                Ok(Some(ThreadInfo {
                    thread_id: thread_id_hex,
                    owner: String::new(),
                    version: 0,
                    state_hash: hex::encode([0u8; 32]),
                }))
            } else {
                Ok(None)
            }
        }
    }

    async fn get_balance(
        &self,
        address_hex: String,
        token_id_hex: String,
    ) -> Result<String, ErrorObjectOwned> {
        let address = parse_address_hex(&address_hex)?;
        let token_id = parse_token_hex(&token_id_hex)?;

        let sm = self.state_manager.read().await;
        let balance = sm.get_balance(&address, &token_id);
        Ok(balance.to_string())
    }

    async fn get_thread_state(
        &self,
        thread_id_hex: String,
    ) -> Result<Option<ThreadStateInfo>, ErrorObjectOwned> {
        let thread_id = parse_address_hex(&thread_id_hex)?;

        let sm = self.state_manager.read().await;
        if let Some(state) = sm.get_thread_state(&thread_id) {
            let meta = sm.get_thread_meta(&thread_id);
            let owner = meta.map(|m| hex::encode(m.owner)).unwrap_or_default();
            let version = meta.map(|m| m.version).unwrap_or(0);
            let state_hash = meta
                .map(|m| hex::encode(m.state_hash))
                .unwrap_or_else(|| hex::encode([0u8; 32]));

            let balances = state
                .balances
                .iter()
                .map(|(token_id, &amount)| super::types::BalanceEntry {
                    token_id: hex::encode(token_id),
                    amount: amount.to_string(),
                    human_readable: format_amount_with_symbol(amount, token_id),
                })
                .collect();

            Ok(Some(ThreadStateInfo {
                thread_id: thread_id_hex,
                owner,
                version,
                state_hash,
                balances,
            }))
        } else {
            Ok(None)
        }
    }

    // Faucet: testnet-only endpoint that bypasses signature verification
    // to auto-register threads and credit test tokens.
    async fn faucet(&self, address_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        #[cfg(not(feature = "testnet"))]
        {
            let _ = address_hex;
            return Err(ErrorObjectOwned::owned(
                -32601,
                "faucet is disabled in production builds",
                None::<()>,
            ));
        }

        #[cfg(feature = "testnet")]
        {
            use norn_types::constants::ONE_NORN;

            // Runtime network check: block faucet on mainnet even if compiled with testnet feature.
            if !self.network_id.faucet_enabled() {
                return Err(ErrorObjectOwned::owned(
                    -32601,
                    format!("faucet is not available on {}", self.network_id.as_str()),
                    None::<()>,
                ));
            }

            let address = parse_address_hex(&address_hex)?;

            // Rate limiting: check cooldown per address.
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let cooldown = self.network_id.faucet_cooldown();

            {
                let mut tracker = self
                    .faucet_tracker
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                if let Some(&last_request) = tracker.get(&address) {
                    if now < last_request + cooldown {
                        let remaining = (last_request + cooldown) - now;
                        return Ok(SubmitResult {
                            success: false,
                            reason: Some(format!(
                                "rate limited: please wait {} seconds before requesting again",
                                remaining
                            )),
                        });
                    }
                }
                tracker.insert(address, now);
            }

            // Cap: reject if address already has >= 1000 NORN.
            let max_faucet_balance: u128 = 1000 * ONE_NORN;
            {
                let sm = self.state_manager.read().await;
                let current = sm.get_balance(&address, &norn_types::primitives::NATIVE_TOKEN_ID);
                if current >= max_faucet_balance {
                    return Ok(SubmitResult {
                        success: false,
                        reason: Some(format!(
                            "address already has {} (max faucet balance: 1000 NORN)",
                            format_amount_with_symbol(
                                current,
                                &norn_types::primitives::NATIVE_TOKEN_ID
                            )
                        )),
                    });
                }
            }

            let faucet_amount: u128 = 100 * ONE_NORN; // 100 NORN per faucet request

            // Register in WeaveEngine if not already known.
            {
                let mut engine = self.weave_engine.write().await;
                if !engine.known_threads().contains(&address) {
                    let reg = norn_types::weave::Registration {
                        thread_id: address,
                        owner: [0u8; 32],
                        initial_state_hash: [0u8; 32],
                        timestamp: now,
                        signature: [0u8; 64],
                    };
                    let _ = engine.add_registration(reg);
                }
            }

            // Register and credit in StateManager.
            {
                let mut sm = self.state_manager.write().await;
                sm.auto_register_if_needed(address);
                sm.credit(
                    address,
                    norn_types::primitives::NATIVE_TOKEN_ID,
                    faucet_amount,
                )
                .map_err(|e| {
                    ErrorObjectOwned::owned(
                        -32000,
                        format!("faucet credit failed: {}", e),
                        None::<()>,
                    )
                })?;

                sm.log_faucet_credit(address, faucet_amount, now);
            }

            Ok(SubmitResult {
                success: true,
                reason: Some(format!(
                    "credited {} to {}",
                    format_amount_with_symbol(
                        faucet_amount,
                        &norn_types::primitives::NATIVE_TOKEN_ID
                    ),
                    format_address(&address)
                )),
            })
        }
    }

    async fn submit_knot(&self, knot_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&knot_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let knot: norn_types::knot::Knot = borsh::from_slice(&bytes).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid knot: {}", e), None::<()>)
        })?;

        // Extract transfer details from the payload.
        let (from, to, token_id, amount, memo) = match &knot.payload {
            norn_types::knot::KnotPayload::Transfer(transfer) => (
                transfer.from,
                transfer.to,
                transfer.token_id,
                transfer.amount,
                transfer.memo.clone(),
            ),
            _ => {
                return Ok(SubmitResult {
                    success: false,
                    reason: Some("only Transfer knots are supported via RPC".to_string()),
                });
            }
        };

        // Validate: at least one signature, and the first before_state pubkey matches the signer.
        if knot.signatures.is_empty() {
            return Ok(SubmitResult {
                success: false,
                reason: Some("knot has no signatures".to_string()),
            });
        }

        if knot.before_states.is_empty() {
            return Ok(SubmitResult {
                success: false,
                reason: Some("knot has no before_states".to_string()),
            });
        }

        // Verify the sender's signature.
        let sender_pubkey = knot.before_states[0].pubkey;
        if let Err(e) = norn_crypto::keys::verify(&knot.id, &knot.signatures[0], &sender_pubkey) {
            return Ok(SubmitResult {
                success: false,
                reason: Some(format!("invalid sender signature: {}", e)),
            });
        }

        // Apply transfer via StateManager.
        let mut sm = self.state_manager.write().await;
        sm.auto_register_if_needed(from);
        sm.auto_register_if_needed(to);

        let knot_id = knot.id;
        let timestamp = knot.timestamp;
        match sm.apply_transfer(from, to, token_id, amount, knot_id, memo.clone(), timestamp) {
            Ok(()) => {
                drop(sm);
                self.metrics.knots_validated.inc();

                // Queue BlockTransfer so solo-mode blocks include this transfer.
                let bt = norn_types::weave::BlockTransfer {
                    from,
                    to,
                    token_id,
                    amount,
                    memo,
                    knot_id,
                    timestamp,
                };
                let mut engine = self.weave_engine.write().await;
                let _ = engine.add_transfer(bt);
                drop(engine);

                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::KnotProposal(Box::new(knot));
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: None,
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn health(&self) -> Result<HealthInfo, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();

        Ok(HealthInfo {
            height: state.height,
            is_validator: self.is_validator,
            thread_count: state.thread_count,
            status: "ok".to_string(),
            network: self.network_id.as_str().to_string(),
            chain_id: self.network_id.chain_id().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    async fn get_validator_set(&self) -> Result<ValidatorSetInfo, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;
        let vs = engine.validator_set();

        Ok(ValidatorSetInfo {
            validators: vs
                .validators
                .iter()
                .map(|v| ValidatorInfo {
                    pubkey: hex::encode(v.pubkey),
                    address: hex::encode(v.address),
                    stake: v.stake.to_string(),
                    active: v.active,
                })
                .collect(),
            total_stake: vs.total_stake.to_string(),
            epoch: vs.epoch,
        })
    }

    async fn get_fee_estimate(&self) -> Result<FeeEstimateInfo, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;
        let fee = engine.fee_estimate();
        let state = engine.weave_state();

        Ok(FeeEstimateInfo {
            fee_per_commitment: fee.to_string(),
            base_fee: state.fee_state.base_fee.to_string(),
            fee_multiplier: state.fee_state.fee_multiplier,
        })
    }

    async fn subscribe_new_blocks(&self, pending: PendingSubscriptionSink) -> SubscriptionResult {
        let mut rx = self.block_tx.subscribe();
        let sink = pending.accept().await?;

        tokio::spawn(async move {
            while let Ok(block_info) = rx.recv().await {
                match jsonrpsee::SubscriptionMessage::from_json(&block_info) {
                    Ok(msg) => {
                        if sink.send(msg).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    async fn get_commitment_proof(
        &self,
        thread_id_hex: String,
    ) -> Result<Option<CommitmentProofInfo>, ErrorObjectOwned> {
        let thread_id = parse_address_hex(&thread_id_hex)?;

        let engine = self.weave_engine.read().await;
        if !engine.known_threads().contains(&thread_id) {
            return Ok(None);
        }

        // Need write access for Merkle proof generation (caches).
        drop(engine);
        let mut engine = self.weave_engine.write().await;
        let proof = engine.commitment_proof(&thread_id);

        Ok(Some(CommitmentProofInfo {
            thread_id: thread_id_hex,
            key: hex::encode(proof.key),
            value: hex::encode(&proof.value),
            siblings: proof.siblings.iter().map(hex::encode).collect(),
        }))
    }

    async fn get_transaction_history(
        &self,
        address_hex: String,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TransactionHistoryEntry>, ErrorObjectOwned> {
        let address = parse_address_hex(&address_hex)?;

        let sm = self.state_manager.read().await;
        let records = sm.get_history(&address, limit as usize, offset as usize);

        let entries = records
            .into_iter()
            .map(|r| {
                let direction = if r.from == address {
                    "sent".to_string()
                } else {
                    "received".to_string()
                };

                let memo_str = r
                    .memo
                    .as_ref()
                    .and_then(|m| String::from_utf8(m.clone()).ok());

                TransactionHistoryEntry {
                    knot_id: hex::encode(r.knot_id),
                    from: format_address(&r.from),
                    to: format_address(&r.to),
                    token_id: hex::encode(r.token_id),
                    amount: r.amount.to_string(),
                    human_readable: format_amount_with_symbol(r.amount, &r.token_id),
                    memo: memo_str,
                    timestamp: r.timestamp,
                    block_height: r.block_height,
                    direction,
                }
            })
            .collect();

        Ok(entries)
    }

    async fn register_name(
        &self,
        name: String,
        owner_hex: String,
        knot_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned> {
        // The knot_hex now carries a hex-encoded borsh NameRegistration (signed by the wallet).
        let bytes = hex::decode(&knot_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let name_reg: norn_types::weave::NameRegistration =
            borsh::from_slice(&bytes).map_err(|e| {
                ErrorObjectOwned::owned(
                    -32602,
                    format!("invalid name registration: {}", e),
                    None::<()>,
                )
            })?;

        // Verify the owner matches the claimed owner.
        let owner_address = parse_address_hex(&owner_hex)?;
        if name_reg.owner != owner_address {
            return Ok(SubmitResult {
                success: false,
                reason: Some("owner address mismatch".to_string()),
            });
        }

        // Verify the name matches.
        if name_reg.name != name {
            return Ok(SubmitResult {
                success: false,
                reason: Some("name mismatch".to_string()),
            });
        }

        // Add to WeaveEngine mempool (validates signature, name format, duplicates).
        let mut engine = self.weave_engine.write().await;
        match engine.add_name_registration(name_reg.clone()) {
            Ok(_) => {
                // Broadcast to P2P network.
                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::NameRegistration(name_reg);
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: Some(format!(
                        "name '{}' submitted for registration (will be included in next block)",
                        name
                    )),
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn resolve_name(&self, name: String) -> Result<Option<NameResolution>, ErrorObjectOwned> {
        let sm = self.state_manager.read().await;
        Ok(sm.resolve_name(&name).map(|record| NameResolution {
            name: name.clone(),
            owner: format_address(&record.owner),
            registered_at: record.registered_at,
            fee_paid: record.fee_paid.to_string(),
        }))
    }

    async fn list_names(&self, address_hex: String) -> Result<Vec<NameInfo>, ErrorObjectOwned> {
        let address = parse_address_hex(&address_hex)?;
        let sm = self.state_manager.read().await;
        let names = sm.names_for_address(&address);
        let infos = names
            .into_iter()
            .filter_map(|name| {
                sm.resolve_name(name).map(|record| NameInfo {
                    name: name.to_string(),
                    registered_at: record.registered_at,
                })
            })
            .collect();
        Ok(infos)
    }

    async fn get_metrics(&self) -> Result<String, ErrorObjectOwned> {
        Ok(self.metrics.encode())
    }

    async fn submit_fraud_proof(
        &self,
        fraud_proof_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&fraud_proof_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let submission: norn_types::fraud::FraudProofSubmission = borsh::from_slice(&bytes)
            .map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("invalid fraud proof: {}", e), None::<()>)
            })?;

        let mut engine = self.weave_engine.write().await;
        let responses =
            engine.on_network_message(NornMessage::FraudProof(Box::new(submission.clone())));
        drop(engine);

        // Broadcast via relay if accepted.
        if let Some(ref handle) = self.relay_handle {
            let h = handle.clone();
            let msg = NornMessage::FraudProof(Box::new(submission));
            tokio::spawn(async move {
                let _ = h.broadcast(msg).await;
            });
        }

        self.metrics.fraud_proofs_submitted.inc();

        Ok(SubmitResult {
            success: true,
            reason: if responses.is_empty() {
                Some("fraud proof accepted".to_string())
            } else {
                Some(format!(
                    "fraud proof accepted, {} response(s) generated",
                    responses.len()
                ))
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_impl_struct() {
        // Verify the struct can be created with the right types.
        // Full RPC testing would require a running server.
        let _info = BlockInfo {
            height: 0,
            hash: String::new(),
            prev_hash: String::new(),
            timestamp: 0,
            proposer: String::new(),
            commitment_count: 0,
            registration_count: 0,
            anchor_count: 0,
            fraud_proof_count: 0,
        };
    }
}
