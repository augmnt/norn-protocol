use std::sync::Arc;
use tokio::sync::RwLock;

use jsonrpsee::core::async_trait;
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::PendingSubscriptionSink;

use norn_types::network::NornMessage;
use norn_weave::engine::WeaveEngine;

use norn_loom::lifecycle::LoomManager;

use super::types::{
    AttributeInfo, BlockInfo, BlockLoomDeployInfo, BlockNameRegistrationInfo, BlockTokenBurnInfo,
    BlockTokenDefinitionInfo, BlockTokenMintInfo, BlockTransactionsInfo, BlockTransferInfo,
    CommitmentProofInfo, EventInfo, ExecutionResult, FeeEstimateInfo, HealthInfo,
    LoomExecutionEvent, LoomInfo, NameInfo, NameResolution, PendingTransactionEvent, QueryResult,
    StakingInfo, StateProofInfo, SubmitResult, ThreadInfo, ThreadStateInfo, TokenEvent, TokenInfo,
    TransactionHistoryEntry, TransferEvent, ValidatorInfo, ValidatorSetInfo, ValidatorStakeInfo,
    WeaveStateInfo,
};
use crate::metrics::NodeMetrics;
use crate::rpc::server::RpcBroadcasters;
use crate::state_manager::StateManager;
use norn_types::constants::{MAX_SUPPLY, NORN_DECIMALS};
use norn_types::primitives::NATIVE_TOKEN_ID;

use crate::wallet::format::{format_address, format_amount_with_symbol, format_token_amount};

/// Format an amount using the correct decimals for the given token.
/// Returns just the numeric string (no symbol) for programmatic use by frontends.
fn format_amount_for_token(amount: u128, token_id: &[u8; 32], sm: &StateManager) -> String {
    if *token_id == NATIVE_TOKEN_ID {
        format_token_amount(amount, NORN_DECIMALS as u8)
    } else if let Some(record) = sm.get_token(token_id) {
        format_token_amount(amount, record.decimals)
    } else {
        // Unknown token — fall back to raw amount.
        amount.to_string()
    }
}

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

    /// Subscribe to transfer events, optionally filtered by address.
    #[subscription(name = "norn_subscribeTransfers" => "norn_transfers", unsubscribe = "norn_unsubscribeTransfers", item = TransferEvent)]
    async fn subscribe_transfers(&self, address_hex: Option<String>) -> SubscriptionResult;

    /// Subscribe to token events (create/mint/burn), optionally filtered by token ID.
    #[subscription(name = "norn_subscribeTokenEvents" => "norn_tokenEvents", unsubscribe = "norn_unsubscribeTokenEvents", item = TokenEvent)]
    async fn subscribe_token_events(&self, token_id_hex: Option<String>) -> SubscriptionResult;

    /// Subscribe to loom execution events, optionally filtered by loom ID.
    #[subscription(name = "norn_subscribeLoomEvents" => "norn_loomEvents", unsubscribe = "norn_unsubscribeLoomEvents", item = LoomExecutionEvent)]
    async fn subscribe_loom_events(&self, loom_id_hex: Option<String>) -> SubscriptionResult;

    /// Subscribe to pending transactions entering the mempool.
    #[subscription(name = "norn_subscribePendingTransactions" => "norn_pendingTransactions", unsubscribe = "norn_unsubscribePendingTransactions", item = PendingTransactionEvent)]
    async fn subscribe_pending_transactions(&self) -> SubscriptionResult;

    /// Get transaction history for an address.
    #[method(name = "norn_getTransactionHistory")]
    async fn get_transaction_history(
        &self,
        address: String,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TransactionHistoryEntry>, ErrorObjectOwned>;

    /// Get recent transactions across all addresses.
    #[method(name = "norn_getRecentTransfers")]
    async fn get_recent_transfers(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TransactionHistoryEntry>, ErrorObjectOwned>;

    /// Get a single transaction by its knot ID (hex).
    #[method(name = "norn_getTransaction")]
    async fn get_transaction(
        &self,
        knot_id: String,
    ) -> Result<Option<TransactionHistoryEntry>, ErrorObjectOwned>;

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

    /// Create a new token (hex-encoded borsh TokenDefinition).
    #[method(name = "norn_createToken")]
    async fn create_token(&self, token_def_hex: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Mint tokens (hex-encoded borsh TokenMint).
    #[method(name = "norn_mintToken")]
    async fn mint_token(&self, token_mint_hex: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Burn tokens (hex-encoded borsh TokenBurn).
    #[method(name = "norn_burnToken")]
    async fn burn_token(&self, token_burn_hex: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Get token info by token ID (hex).
    #[method(name = "norn_getTokenInfo")]
    async fn get_token_info(
        &self,
        token_id_hex: String,
    ) -> Result<Option<TokenInfo>, ErrorObjectOwned>;

    /// Get token info by symbol.
    #[method(name = "norn_getTokenBySymbol")]
    async fn get_token_by_symbol(
        &self,
        symbol: String,
    ) -> Result<Option<TokenInfo>, ErrorObjectOwned>;

    /// List all tokens with pagination.
    #[method(name = "norn_listTokens")]
    async fn list_tokens(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TokenInfo>, ErrorObjectOwned>;

    /// Deploy a loom (hex-encoded borsh LoomRegistration).
    #[method(name = "norn_deployLoom")]
    async fn deploy_loom(&self, deploy_hex: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Get loom info by loom ID (hex).
    #[method(name = "norn_getLoomInfo")]
    async fn get_loom_info(
        &self,
        loom_id_hex: String,
    ) -> Result<Option<LoomInfo>, ErrorObjectOwned>;

    /// List all deployed looms with pagination.
    #[method(name = "norn_listLooms")]
    async fn list_looms(&self, limit: u64, offset: u64) -> Result<Vec<LoomInfo>, ErrorObjectOwned>;

    /// Upload bytecode to a deployed loom and initialize it.
    /// Optionally pass init_msg_hex for typed constructor parameters.
    #[method(name = "norn_uploadLoomBytecode")]
    async fn upload_loom_bytecode(
        &self,
        loom_id_hex: String,
        bytecode_hex: String,
        init_msg_hex: Option<String>,
    ) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Execute a loom contract (state-mutating).
    #[method(name = "norn_executeLoom")]
    async fn execute_loom(
        &self,
        loom_id_hex: String,
        input_hex: String,
        sender_hex: String,
    ) -> Result<ExecutionResult, ErrorObjectOwned>;

    /// Query a loom contract (read-only).
    #[method(name = "norn_queryLoom")]
    async fn query_loom(
        &self,
        loom_id_hex: String,
        input_hex: String,
    ) -> Result<QueryResult, ErrorObjectOwned>;

    /// Join a loom as a participant.
    #[method(name = "norn_joinLoom")]
    async fn join_loom(
        &self,
        loom_id_hex: String,
        participant_hex: String,
        pubkey_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Leave a loom.
    #[method(name = "norn_leaveLoom")]
    async fn leave_loom(
        &self,
        loom_id_hex: String,
        participant_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Submit a stake operation (hex-encoded borsh StakeOperation).
    #[method(name = "norn_stake")]
    async fn stake(&self, operation_hex: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Submit an unstake operation (hex-encoded borsh StakeOperation).
    #[method(name = "norn_unstake")]
    async fn unstake(&self, operation_hex: String) -> Result<SubmitResult, ErrorObjectOwned>;

    /// Get staking info (all validators or specific).
    #[method(name = "norn_getStakingInfo")]
    async fn get_staking_info(
        &self,
        pubkey_hex: Option<String>,
    ) -> Result<StakingInfo, ErrorObjectOwned>;

    /// Get the current state root.
    #[method(name = "norn_getStateRoot")]
    async fn get_state_root(&self) -> Result<String, ErrorObjectOwned>;

    /// Get a state proof for a balance.
    #[method(name = "norn_getStateProof")]
    async fn get_state_proof(
        &self,
        address_hex: String,
        token_id_hex: Option<String>,
    ) -> Result<StateProofInfo, ErrorObjectOwned>;

    /// Get detailed transactions for a block by height.
    #[method(name = "norn_getBlockTransactions")]
    async fn get_block_transactions(
        &self,
        height: u64,
    ) -> Result<Option<BlockTransactionsInfo>, ErrorObjectOwned>;
}

/// Implementation of the NornRpc trait.
#[allow(dead_code)] // Required: jsonrpsee accesses fields via trait impl
pub struct NornRpcImpl {
    pub weave_engine: Arc<RwLock<WeaveEngine>>,
    pub state_manager: Arc<RwLock<StateManager>>,
    pub loom_manager: Arc<RwLock<LoomManager>>,
    pub metrics: Arc<NodeMetrics>,
    pub broadcasters: RpcBroadcasters,
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

/// Parse a hex-encoded loom ID into a [u8; 32].
fn parse_loom_hex(hex_str: &str) -> Result<[u8; 32], ErrorObjectOwned> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>))?;
    if bytes.len() != 32 {
        return Err(ErrorObjectOwned::owned(
            -32602,
            format!("loom_id must be 32 bytes, got {}", bytes.len()),
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
        // Try the StateManager archive (in-memory + SQLite fallback).
        let sm = self.state_manager.read().await;
        if let Some(block) = sm.get_block_by_height(height) {
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
                name_registration_count: block.name_registrations.len(),
                transfer_count: block.transfers.len(),
                token_definition_count: block.token_definitions.len(),
                token_mint_count: block.token_mints.len(),
                token_burn_count: block.token_burns.len(),
                loom_deploy_count: block.loom_deploys.len(),
                stake_operation_count: block.stake_operations.len(),
                state_root: hex::encode(block.state_root),
            }));
        }

        Ok(None)
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
                name_registration_count: block.name_registrations.len(),
                transfer_count: block.transfers.len(),
                token_definition_count: block.token_definitions.len(),
                token_mint_count: block.token_mints.len(),
                token_burn_count: block.token_burns.len(),
                loom_deploy_count: block.loom_deploys.len(),
                stake_operation_count: block.stake_operations.len(),
                state_root: hex::encode(block.state_root),
            }))
        } else {
            let height = engine.weave_state().height;
            drop(engine);

            // Try the StateManager archive (in-memory + SQLite fallback).
            let sm = self.state_manager.read().await;
            if let Some(block) = sm.get_block_by_height(height) {
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
                    name_registration_count: block.name_registrations.len(),
                    transfer_count: block.transfers.len(),
                    token_definition_count: block.token_definitions.len(),
                    token_mint_count: block.token_mints.len(),
                    token_burn_count: block.token_burns.len(),
                    loom_deploy_count: block.loom_deploys.len(),
                    stake_operation_count: block.stake_operations.len(),
                    state_root: hex::encode(block.state_root),
                }));
            }

            Ok(None)
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
                    human_readable: format_amount_for_token(amount, token_id, &sm),
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

            let faucet_amount: u128 = 100 * ONE_NORN; // 100 NORN per faucet request
            let faucet_address: [u8; 20] = [0u8; 20]; // Zero address = faucet source

            // Deterministic knot_id for dedup across nodes.
            let knot_id = {
                let mut data = Vec::with_capacity(6 + 20 + 8);
                data.extend_from_slice(b"faucet");
                data.extend_from_slice(&address);
                data.extend_from_slice(&now.to_le_bytes());
                norn_crypto::hash::blake3_hash(&data)
            };

            // Register recipient in WeaveEngine if not already known.
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

            // Balance check + credit under a single write lock to prevent TOCTOU race.
            {
                let mut sm = self.state_manager.write().await;

                // Cap: reject if address already has >= 1000 NORN.
                let max_faucet_balance: u128 = 1000 * ONE_NORN;
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

                sm.auto_register_if_needed(faucet_address);
                sm.auto_register_if_needed(address);
                if let Err(e) = sm.apply_peer_transfer(
                    faucet_address,
                    address,
                    norn_types::primitives::NATIVE_TOKEN_ID,
                    faucet_amount,
                    knot_id,
                    Some(b"faucet".to_vec()),
                    now,
                ) {
                    return Err(ErrorObjectOwned::owned(
                        -32000,
                        format!("faucet credit failed: {}", e),
                        None::<()>,
                    ));
                }
            }

            // Queue BlockTransfer for inclusion in the next block.
            let bt = norn_types::weave::BlockTransfer {
                from: faucet_address,
                to: address,
                token_id: norn_types::primitives::NATIVE_TOKEN_ID,
                amount: faucet_amount,
                memo: Some(b"faucet".to_vec()),
                knot_id,
                timestamp: now,
            };
            {
                let mut engine = self.weave_engine.write().await;
                let _ = engine.add_transfer(bt);
            }

            // Fire pending transaction event for real-time subscribers.
            let _ = self.broadcasters.pending_tx.send(PendingTransactionEvent {
                tx_type: "faucet".to_string(),
                hash: hex::encode(knot_id),
                from: format_address(&faucet_address),
                timestamp: now,
            });

            // Fire transfer event for real-time subscribers.
            let _ = self.broadcasters.transfer_tx.send(TransferEvent {
                from: format_address(&faucet_address),
                to: format_address(&address),
                amount: faucet_amount.to_string(),
                token_id: None,
                symbol: Some("NORN".to_string()),
                memo: Some("faucet".to_string()),
                block_height: None, // Pending — not yet in a block.
            });

            // Gossip faucet credit to peers so the block producer can include it.
            if let Some(ref handle) = self.relay_handle {
                tracing::info!(recipient = %address_hex, "broadcasting faucet credit to peers");
                let h = handle.clone();
                let msg = NornMessage::FaucetCredit(norn_types::network::FaucetCredit {
                    recipient: address,
                    amount: faucet_amount,
                    timestamp: now,
                    knot_id,
                });
                tokio::spawn(async move {
                    let _ = h.broadcast(msg).await;
                });
            } else {
                tracing::warn!("no relay handle — faucet credit not gossiped to peers");
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

        // Verify all signatures match their corresponding before_state pubkeys.
        if knot.signatures.len() != knot.before_states.len() {
            return Ok(SubmitResult {
                success: false,
                reason: Some(format!(
                    "signature count ({}) does not match before_states count ({})",
                    knot.signatures.len(),
                    knot.before_states.len()
                )),
            });
        }

        for (i, (sig, bs)) in knot
            .signatures
            .iter()
            .zip(knot.before_states.iter())
            .enumerate()
        {
            if let Err(e) = norn_crypto::keys::verify(&knot.id, sig, &bs.pubkey) {
                return Ok(SubmitResult {
                    success: false,
                    reason: Some(format!("invalid signature at index {}: {}", i, e)),
                });
            }
        }

        // Apply transfer via StateManager.
        let sender_pubkey = knot.before_states[0].pubkey;
        let mut sm = self.state_manager.write().await;
        sm.auto_register_with_pubkey(from, sender_pubkey);
        sm.auto_register_if_needed(to);

        let knot_id = knot.id;
        let timestamp = knot.timestamp;
        match sm.apply_transfer(from, to, token_id, amount, knot_id, memo.clone(), timestamp) {
            Ok(()) => {
                let token_symbol = if token_id == NATIVE_TOKEN_ID {
                    "NORN".to_string()
                } else {
                    sm.get_token(&token_id)
                        .map(|t| t.symbol.clone())
                        .unwrap_or_else(|| hex::encode(&token_id[..4]))
                };
                drop(sm);
                self.metrics.knots_validated.inc();

                // Fire pending transaction event.
                let _ = self.broadcasters.pending_tx.send(PendingTransactionEvent {
                    tx_type: "transfer".to_string(),
                    hash: hex::encode(knot_id),
                    from: format_address(&from),
                    timestamp,
                });

                // Queue BlockTransfer so solo-mode blocks include this transfer.
                let bt = norn_types::weave::BlockTransfer {
                    from,
                    to,
                    token_id,
                    amount,
                    memo: memo.clone(),
                    knot_id,
                    timestamp,
                };
                let mut engine = self.weave_engine.write().await;
                let _ = engine.add_transfer(bt);
                drop(engine);

                // Fire transfer event for subscribers.
                let native = norn_types::primitives::NATIVE_TOKEN_ID;
                let _ = self.broadcasters.transfer_tx.send(TransferEvent {
                    from: format_address(&from),
                    to: format_address(&to),
                    amount: amount.to_string(),
                    token_id: if token_id == native {
                        None
                    } else {
                        Some(hex::encode(token_id))
                    },
                    symbol: Some(token_symbol),
                    memo: memo
                        .as_ref()
                        .and_then(|m| String::from_utf8(m.clone()).ok()),
                    block_height: None, // Pending — not yet in a block.
                });

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
        let mut rx = self.broadcasters.block_tx.subscribe();
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

    async fn subscribe_transfers(
        &self,
        pending: PendingSubscriptionSink,
        address_hex: Option<String>,
    ) -> SubscriptionResult {
        let mut rx = self.broadcasters.transfer_tx.subscribe();
        let sink = pending.accept().await?;
        let filter_addr = address_hex.clone();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                // Apply optional address filter.
                if let Some(ref addr) = filter_addr {
                    if event.from != *addr && event.to != *addr {
                        continue;
                    }
                }
                match jsonrpsee::SubscriptionMessage::from_json(&event) {
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

    async fn subscribe_token_events(
        &self,
        pending: PendingSubscriptionSink,
        token_id_hex: Option<String>,
    ) -> SubscriptionResult {
        let mut rx = self.broadcasters.token_tx.subscribe();
        let sink = pending.accept().await?;
        let filter_token = token_id_hex.clone();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if let Some(ref tid) = filter_token {
                    if event.token_id != *tid {
                        continue;
                    }
                }
                match jsonrpsee::SubscriptionMessage::from_json(&event) {
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

    async fn subscribe_loom_events(
        &self,
        pending: PendingSubscriptionSink,
        loom_id_hex: Option<String>,
    ) -> SubscriptionResult {
        let mut rx = self.broadcasters.loom_tx.subscribe();
        let sink = pending.accept().await?;
        let filter_loom = loom_id_hex.clone();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if let Some(ref lid) = filter_loom {
                    if event.loom_id != *lid {
                        continue;
                    }
                }
                match jsonrpsee::SubscriptionMessage::from_json(&event) {
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

    async fn subscribe_pending_transactions(
        &self,
        pending: PendingSubscriptionSink,
    ) -> SubscriptionResult {
        let mut rx = self.broadcasters.pending_tx.subscribe();
        let sink = pending.accept().await?;

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                match jsonrpsee::SubscriptionMessage::from_json(&event) {
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

        // Cap limit to prevent excessive memory use.
        let limit = if limit == 0 { 100 } else { limit.min(1000) } as usize;
        let offset = offset as usize;

        let sm = self.state_manager.read().await;
        let records = sm.get_history(&address, limit, offset);

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
                    symbol: if r.token_id == NATIVE_TOKEN_ID {
                        "NORN".to_string()
                    } else {
                        sm.get_token(&r.token_id)
                            .map(|t| t.symbol.clone())
                            .unwrap_or_else(|| hex::encode(&r.token_id[..4]))
                    },
                    amount: r.amount.to_string(),
                    human_readable: format_amount_for_token(r.amount, &r.token_id, &sm),
                    memo: memo_str,
                    timestamp: r.timestamp,
                    block_height: r.block_height,
                    direction,
                }
            })
            .collect();

        Ok(entries)
    }

    async fn get_recent_transfers(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TransactionHistoryEntry>, ErrorObjectOwned> {
        let limit = if limit == 0 { 20 } else { limit.min(100) } as usize;
        let offset = offset as usize;

        let sm = self.state_manager.read().await;
        let records = sm.get_recent_transfers(limit, offset);

        let entries = records
            .into_iter()
            .map(|r| TransactionHistoryEntry {
                knot_id: hex::encode(r.knot_id),
                from: format_address(&r.from),
                to: format_address(&r.to),
                token_id: hex::encode(r.token_id),
                symbol: if r.token_id == NATIVE_TOKEN_ID {
                    "NORN".to_string()
                } else {
                    sm.get_token(&r.token_id)
                        .map(|t| t.symbol.clone())
                        .unwrap_or_else(|| hex::encode(&r.token_id[..4]))
                },
                amount: r.amount.to_string(),
                human_readable: format_amount_for_token(r.amount, &r.token_id, &sm),
                memo: r
                    .memo
                    .as_ref()
                    .and_then(|m| String::from_utf8(m.clone()).ok()),
                timestamp: r.timestamp,
                block_height: r.block_height,
                direction: String::new(),
            })
            .collect();

        Ok(entries)
    }

    async fn get_transaction(
        &self,
        knot_id: String,
    ) -> Result<Option<TransactionHistoryEntry>, ErrorObjectOwned> {
        let knot_bytes: [u8; 32] = hex::decode(&knot_id)
            .map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
            })?
            .try_into()
            .map_err(|_| ErrorObjectOwned::owned(-32602, "knot_id must be 32 bytes", None::<()>))?;

        let sm = self.state_manager.read().await;
        let entry = sm
            .get_transfer_by_knot_id(&knot_bytes)
            .map(|r| TransactionHistoryEntry {
                knot_id: hex::encode(r.knot_id),
                from: format_address(&r.from),
                to: format_address(&r.to),
                token_id: hex::encode(r.token_id),
                symbol: if r.token_id == NATIVE_TOKEN_ID {
                    "NORN".to_string()
                } else {
                    sm.get_token(&r.token_id)
                        .map(|t| t.symbol.clone())
                        .unwrap_or_else(|| hex::encode(&r.token_id[..4]))
                },
                amount: r.amount.to_string(),
                human_readable: format_amount_for_token(r.amount, &r.token_id, &sm),
                memo: r
                    .memo
                    .as_ref()
                    .and_then(|m| String::from_utf8(m.clone()).ok()),
                timestamp: r.timestamp,
                block_height: r.block_height,
                direction: String::new(),
            });

        Ok(entry)
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

        // Validate submitter pubkey is not zero.
        if submission.submitter == [0u8; 32] {
            return Ok(SubmitResult {
                success: false,
                reason: Some("submitter public key must not be zero".to_string()),
            });
        }

        // Verify the submitter's signature over the proof.
        let proof_bytes = borsh::to_vec(&submission.proof).map_err(|e| {
            ErrorObjectOwned::owned(
                -32000,
                format!("failed to serialize proof for verification: {}", e),
                None::<()>,
            )
        })?;
        let proof_hash = norn_crypto::hash::blake3_hash(&proof_bytes);
        if let Err(e) =
            norn_crypto::keys::verify(&proof_hash, &submission.signature, &submission.submitter)
        {
            return Ok(SubmitResult {
                success: false,
                reason: Some(format!("invalid submitter signature: {}", e)),
            });
        }

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

    async fn create_token(&self, token_def_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&token_def_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let token_def: norn_types::weave::TokenDefinition =
            borsh::from_slice(&bytes).map_err(|e| {
                ErrorObjectOwned::owned(
                    -32602,
                    format!("invalid token definition: {}", e),
                    None::<()>,
                )
            })?;

        // Add to WeaveEngine mempool (validates signature, symbol, duplicates).
        let mut engine = self.weave_engine.write().await;
        let block_height = engine.weave_state().height;
        match engine.add_token_definition(token_def.clone()) {
            Ok(_) => {
                // Fire token event.
                let tid = norn_types::token::compute_token_id(
                    &token_def.creator,
                    &token_def.name,
                    &token_def.symbol,
                    token_def.decimals,
                    token_def.max_supply,
                    token_def.timestamp,
                );
                let _ = self.broadcasters.token_tx.send(TokenEvent {
                    event_type: "created".to_string(),
                    token_id: hex::encode(tid),
                    symbol: token_def.symbol.clone(),
                    actor: format_address(&token_def.creator),
                    amount: None,
                    block_height,
                });
                let _ = self.broadcasters.pending_tx.send(PendingTransactionEvent {
                    tx_type: "token_create".to_string(),
                    hash: hex::encode(tid),
                    from: format_address(&token_def.creator),
                    timestamp: token_def.timestamp,
                });
                // Broadcast to P2P network.
                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::TokenDefinition(token_def);
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: Some(
                        "token definition submitted (will be included in next block)".to_string(),
                    ),
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn mint_token(&self, token_mint_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&token_mint_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let token_mint: norn_types::weave::TokenMint = borsh::from_slice(&bytes).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid token mint: {}", e), None::<()>)
        })?;

        // Add to WeaveEngine mempool (validates authority, supply cap, etc.).
        let mut engine = self.weave_engine.write().await;
        let block_height = engine.weave_state().height;
        match engine.add_token_mint(token_mint.clone()) {
            Ok(_) => {
                // Fire token event.
                let sm = self.state_manager.read().await;
                let symbol = sm
                    .get_token(&token_mint.token_id)
                    .map(|r| r.symbol.clone())
                    .unwrap_or_default();
                drop(sm);
                let _ = self.broadcasters.token_tx.send(TokenEvent {
                    event_type: "minted".to_string(),
                    token_id: hex::encode(token_mint.token_id),
                    symbol,
                    actor: format_address(&token_mint.to),
                    amount: Some(token_mint.amount.to_string()),
                    block_height,
                });
                // Broadcast to P2P network.
                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::TokenMint(token_mint);
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: Some(
                        "token mint submitted (will be included in next block)".to_string(),
                    ),
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn burn_token(&self, token_burn_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&token_burn_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let token_burn: norn_types::weave::TokenBurn = borsh::from_slice(&bytes).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid token burn: {}", e), None::<()>)
        })?;

        // Add to WeaveEngine mempool (validates signature, token exists, etc.).
        let mut engine = self.weave_engine.write().await;
        let block_height = engine.weave_state().height;
        match engine.add_token_burn(token_burn.clone()) {
            Ok(_) => {
                // Fire token event.
                let sm = self.state_manager.read().await;
                let symbol = sm
                    .get_token(&token_burn.token_id)
                    .map(|r| r.symbol.clone())
                    .unwrap_or_default();
                drop(sm);
                let _ = self.broadcasters.token_tx.send(TokenEvent {
                    event_type: "burned".to_string(),
                    token_id: hex::encode(token_burn.token_id),
                    symbol,
                    actor: format_address(&token_burn.burner),
                    amount: Some(token_burn.amount.to_string()),
                    block_height,
                });
                // Broadcast to P2P network.
                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::TokenBurn(token_burn);
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: Some(
                        "token burn submitted (will be included in next block)".to_string(),
                    ),
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn get_token_info(
        &self,
        token_id_hex: String,
    ) -> Result<Option<TokenInfo>, ErrorObjectOwned> {
        let token_id = parse_token_hex(&token_id_hex)?;

        // Native NORN token is not in the token registry — synthesize its info.
        if token_id == NATIVE_TOKEN_ID {
            let sm = self.state_manager.read().await;
            return Ok(Some(TokenInfo {
                token_id: token_id_hex,
                name: "Norn".to_string(),
                symbol: "NORN".to_string(),
                decimals: NORN_DECIMALS as u8,
                max_supply: MAX_SUPPLY.to_string(),
                current_supply: sm.total_supply().to_string(),
                creator: format_address(&[0u8; 20]),
                created_at: 0,
            }));
        }

        let sm = self.state_manager.read().await;
        Ok(sm.get_token(&token_id).map(|record| TokenInfo {
            token_id: token_id_hex,
            name: record.name.clone(),
            symbol: record.symbol.clone(),
            decimals: record.decimals,
            max_supply: record.max_supply.to_string(),
            current_supply: record.current_supply.to_string(),
            creator: format_address(&record.creator),
            created_at: record.created_at,
        }))
    }

    async fn get_token_by_symbol(
        &self,
        symbol: String,
    ) -> Result<Option<TokenInfo>, ErrorObjectOwned> {
        let sm = self.state_manager.read().await;
        let (token_id, record) = match sm.get_token_by_symbol(&symbol) {
            Some(pair) => pair,
            None => return Ok(None),
        };
        Ok(Some(TokenInfo {
            token_id: hex::encode(token_id),
            name: record.name.clone(),
            symbol: record.symbol.clone(),
            decimals: record.decimals,
            max_supply: record.max_supply.to_string(),
            current_supply: record.current_supply.to_string(),
            creator: format_address(&record.creator),
            created_at: record.created_at,
        }))
    }

    async fn list_tokens(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TokenInfo>, ErrorObjectOwned> {
        let limit = if limit == 0 { 50 } else { limit.min(200) } as usize;
        let offset = offset as usize;

        let sm = self.state_manager.read().await;

        // Synthesize native NORN token as the first entry.
        let native = TokenInfo {
            token_id: hex::encode(NATIVE_TOKEN_ID),
            name: "Norn".to_string(),
            symbol: "NORN".to_string(),
            decimals: NORN_DECIMALS as u8,
            max_supply: MAX_SUPPLY.to_string(),
            current_supply: sm.total_supply().to_string(),
            creator: format_address(&[0u8; 20]),
            created_at: 0,
        };

        let user_tokens = sm.list_tokens();

        let result = std::iter::once(native)
            .chain(user_tokens.into_iter().map(|(token_id, record)| TokenInfo {
                token_id: hex::encode(token_id),
                name: record.name.clone(),
                symbol: record.symbol.clone(),
                decimals: record.decimals,
                max_supply: record.max_supply.to_string(),
                current_supply: record.current_supply.to_string(),
                creator: format_address(&record.creator),
                created_at: record.created_at,
            }))
            .skip(offset)
            .take(limit)
            .collect();

        Ok(result)
    }

    async fn deploy_loom(&self, deploy_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&deploy_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        let loom_reg: norn_types::loom::LoomRegistration =
            borsh::from_slice(&bytes).map_err(|e| {
                ErrorObjectOwned::owned(
                    -32602,
                    format!("invalid loom registration: {}", e),
                    None::<()>,
                )
            })?;

        // Add to WeaveEngine mempool (validates signature, config, duplicates).
        let mut engine = self.weave_engine.write().await;
        match engine.add_loom_deploy(loom_reg.clone()) {
            Ok(loom_id) => {
                // Broadcast to P2P network.
                if let Some(ref handle) = self.relay_handle {
                    let h = handle.clone();
                    let msg = NornMessage::LoomDeploy(Box::new(loom_reg));
                    tokio::spawn(async move {
                        let _ = h.broadcast(msg).await;
                    });
                }
                Ok(SubmitResult {
                    success: true,
                    reason: Some(format!(
                        "loom deployed (id: {}, will be included in next block)",
                        hex::encode(loom_id)
                    )),
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn get_loom_info(
        &self,
        loom_id_hex: String,
    ) -> Result<Option<LoomInfo>, ErrorObjectOwned> {
        let loom_id = parse_loom_hex(&loom_id_hex)?;
        let sm = self.state_manager.read().await;
        let loom_mgr = self.loom_manager.read().await;
        Ok(sm.get_loom(&loom_id).map(|record| LoomInfo {
            loom_id: loom_id_hex,
            name: record.name.clone(),
            operator: hex::encode(record.operator),
            active: record.active,
            deployed_at: record.deployed_at,
            has_bytecode: loom_mgr.has_bytecode(&loom_id),
            participant_count: loom_mgr.participant_count(&loom_id),
        }))
    }

    async fn list_looms(&self, limit: u64, offset: u64) -> Result<Vec<LoomInfo>, ErrorObjectOwned> {
        let limit = if limit == 0 { 50 } else { limit.min(200) } as usize;
        let offset = offset as usize;

        let sm = self.state_manager.read().await;
        let loom_mgr = self.loom_manager.read().await;
        let looms = sm.list_looms();

        let result = looms
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(loom_id, record)| LoomInfo {
                loom_id: hex::encode(loom_id),
                name: record.name.clone(),
                operator: hex::encode(record.operator),
                active: record.active,
                deployed_at: record.deployed_at,
                has_bytecode: loom_mgr.has_bytecode(loom_id),
                participant_count: loom_mgr.participant_count(loom_id),
            })
            .collect();

        Ok(result)
    }

    async fn upload_loom_bytecode(
        &self,
        loom_id_hex: String,
        bytecode_hex: String,
        init_msg_hex: Option<String>,
    ) -> Result<SubmitResult, ErrorObjectOwned> {
        let loom_id = parse_loom_hex(&loom_id_hex)?;
        let bytecode = hex::decode(&bytecode_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;
        let init_msg = match init_msg_hex {
            Some(hex_str) => Some(hex::decode(&hex_str).map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("invalid init_msg hex: {}", e), None::<()>)
            })?),
            None => None,
        };

        // Verify loom exists in StateManager.
        {
            let sm = self.state_manager.read().await;
            if sm.get_loom(&loom_id).is_none() {
                return Ok(SubmitResult {
                    success: false,
                    reason: Some(format!("loom {} not found", loom_id_hex)),
                });
            }
        }

        let mut loom_mgr = self.loom_manager.write().await;
        match loom_mgr.upload_bytecode(&loom_id, bytecode.clone(), init_msg) {
            Ok(()) => {
                // Persist bytecode and initial state.
                let sm = self.state_manager.read().await;
                if let Some(store) = sm.store() {
                    if let Err(e) = store.save_loom_bytecode(&loom_id, &bytecode) {
                        tracing::warn!("failed to persist loom bytecode: {}", e);
                    }
                    if let Some(state_data) = loom_mgr.get_state_data(&loom_id) {
                        let state_bytes = borsh::to_vec(state_data).unwrap_or_default();
                        if let Err(e) = store.save_loom_state(&loom_id, &state_bytes) {
                            tracing::warn!("failed to persist loom state: {}", e);
                        }
                    }
                }

                Ok(SubmitResult {
                    success: true,
                    reason: Some("bytecode uploaded and initialized".to_string()),
                })
            }
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn execute_loom(
        &self,
        loom_id_hex: String,
        input_hex: String,
        sender_hex: String,
    ) -> Result<ExecutionResult, ErrorObjectOwned> {
        let loom_id = parse_loom_hex(&loom_id_hex)?;
        let input = hex::decode(&input_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid input hex: {}", e), None::<()>)
        })?;
        let sender = parse_address_hex(&sender_hex)?;

        // Get current block context.
        let (block_height, timestamp) = {
            let engine = self.weave_engine.read().await;
            let state = engine.weave_state();
            (
                state.height,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            )
        };

        let mut loom_mgr = self.loom_manager.write().await;
        match loom_mgr.execute(&loom_id, &input, sender, block_height, timestamp) {
            Ok(outcome) => {
                // Persist updated state.
                let mut sm = self.state_manager.write().await;
                if let Some(store) = sm.store() {
                    if let Some(state_data) = loom_mgr.get_state_data(&loom_id) {
                        let state_bytes = borsh::to_vec(state_data).unwrap_or_default();
                        if let Err(e) = store.save_loom_state(&loom_id, &state_bytes) {
                            tracing::warn!("failed to persist loom state: {}", e);
                        }
                    }
                }

                // Apply pending transfers to account balances.
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                for pt in &outcome.pending_transfers {
                    sm.auto_register_if_needed(pt.from);
                    sm.auto_register_if_needed(pt.to);
                    if let Err(e) = sm.apply_transfer(
                        pt.from,
                        pt.to,
                        pt.token_id,
                        pt.amount,
                        [0u8; 32], // synthetic knot_id for loom transfers
                        None,
                        now,
                    ) {
                        tracing::warn!(
                            "failed to apply loom transfer from {:?} to {:?}: {}",
                            pt.from,
                            pt.to,
                            e
                        );
                    }
                }

                // Build event info for response.
                let events: Vec<EventInfo> = outcome
                    .events
                    .iter()
                    .map(|e| EventInfo {
                        ty: e.ty.clone(),
                        attributes: e
                            .attributes
                            .iter()
                            .map(|(k, v)| AttributeInfo {
                                key: k.clone(),
                                value: v.clone(),
                            })
                            .collect(),
                    })
                    .collect();

                // Fire loom execution event for subscribers.
                let _ = self.broadcasters.loom_tx.send(LoomExecutionEvent {
                    loom_id: loom_id_hex.clone(),
                    caller: sender_hex.clone(),
                    gas_used: outcome.gas_used,
                    events: events.clone(),
                    block_height,
                });

                Ok(ExecutionResult {
                    success: true,
                    output_hex: Some(hex::encode(&outcome.transition.outputs)),
                    gas_used: outcome.gas_used,
                    logs: outcome.logs,
                    events,
                    reason: None,
                })
            }
            Err(e) => Ok(ExecutionResult {
                success: false,
                output_hex: None,
                gas_used: 0,
                logs: Vec::new(),
                events: Vec::new(),
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn query_loom(
        &self,
        loom_id_hex: String,
        input_hex: String,
    ) -> Result<QueryResult, ErrorObjectOwned> {
        let loom_id = parse_loom_hex(&loom_id_hex)?;
        let input = hex::decode(&input_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid input hex: {}", e), None::<()>)
        })?;

        let (block_height, timestamp) = {
            let engine = self.weave_engine.read().await;
            let state = engine.weave_state();
            (
                state.height,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            )
        };

        let loom_mgr = self.loom_manager.read().await;
        match loom_mgr.query(&loom_id, &input, [0u8; 20], block_height, timestamp) {
            Ok(outcome) => {
                let events: Vec<EventInfo> = outcome
                    .events
                    .iter()
                    .map(|e| EventInfo {
                        ty: e.ty.clone(),
                        attributes: e
                            .attributes
                            .iter()
                            .map(|(k, v)| AttributeInfo {
                                key: k.clone(),
                                value: v.clone(),
                            })
                            .collect(),
                    })
                    .collect();
                Ok(QueryResult {
                    success: true,
                    output_hex: Some(hex::encode(&outcome.output)),
                    gas_used: outcome.gas_used,
                    logs: outcome.logs,
                    events,
                    reason: None,
                })
            }
            Err(e) => Ok(QueryResult {
                success: false,
                output_hex: None,
                gas_used: 0,
                logs: Vec::new(),
                events: Vec::new(),
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn join_loom(
        &self,
        loom_id_hex: String,
        participant_hex: String,
        pubkey_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned> {
        let loom_id = parse_loom_hex(&loom_id_hex)?;
        let address = parse_address_hex(&participant_hex)?;
        let pubkey_bytes = hex::decode(&pubkey_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid pubkey hex: {}", e), None::<()>)
        })?;
        if pubkey_bytes.len() != 32 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                format!("pubkey must be 32 bytes, got {}", pubkey_bytes.len()),
                None::<()>,
            ));
        }
        let mut pubkey = [0u8; 32];
        pubkey.copy_from_slice(&pubkey_bytes);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut loom_mgr = self.loom_manager.write().await;
        match loom_mgr.join(&loom_id, pubkey, address, timestamp) {
            Ok(()) => Ok(SubmitResult {
                success: true,
                reason: Some("joined loom".to_string()),
            }),
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn leave_loom(
        &self,
        loom_id_hex: String,
        participant_hex: String,
    ) -> Result<SubmitResult, ErrorObjectOwned> {
        let loom_id = parse_loom_hex(&loom_id_hex)?;
        let address = parse_address_hex(&participant_hex)?;

        let mut loom_mgr = self.loom_manager.write().await;
        match loom_mgr.leave(&loom_id, &address) {
            Ok(()) => Ok(SubmitResult {
                success: true,
                reason: Some("left loom".to_string()),
            }),
            Err(e) => Ok(SubmitResult {
                success: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn stake(&self, operation_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        let bytes = hex::decode(&operation_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;
        let op: norn_types::weave::StakeOperation = borsh::from_slice(&bytes).map_err(|e| {
            ErrorObjectOwned::owned(
                -32602,
                format!("invalid stake operation: {}", e),
                None::<()>,
            )
        })?;

        // Validate.
        {
            let engine = self.weave_engine.read().await;
            if let Err(e) = norn_weave::staking::validate_stake_operation(&op, engine.staking()) {
                return Ok(SubmitResult {
                    success: false,
                    reason: Some(e.to_string()),
                });
            }
        }

        // Add to mempool.
        {
            let mut engine = self.weave_engine.write().await;
            let _ = engine.mempool_mut().add_stake_operation(op.clone());
        }

        // Fire pending transaction event.
        let (stake_pubkey, stake_timestamp) = match &op {
            norn_types::weave::StakeOperation::Stake {
                pubkey, timestamp, ..
            } => (*pubkey, *timestamp),
            norn_types::weave::StakeOperation::Unstake {
                pubkey, timestamp, ..
            } => (*pubkey, *timestamp),
        };
        let _ = self.broadcasters.pending_tx.send(PendingTransactionEvent {
            tx_type: "stake".to_string(),
            hash: hex::encode(norn_crypto::hash::blake3_hash(&bytes)),
            from: format_address(&norn_crypto::address::pubkey_to_address(&stake_pubkey)),
            timestamp: stake_timestamp,
        });

        // Broadcast via P2P.
        if let Some(ref handle) = self.relay_handle {
            let h = handle.clone();
            let msg = NornMessage::StakeOperation(op);
            tokio::spawn(async move {
                let _ = h.broadcast(msg).await;
            });
        }

        Ok(SubmitResult {
            success: true,
            reason: Some("stake operation submitted".to_string()),
        })
    }

    async fn unstake(&self, operation_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        // Unstake uses the same code path as stake — both are StakeOperation variants.
        self.stake(operation_hex).await
    }

    async fn get_staking_info(
        &self,
        pubkey_hex: Option<String>,
    ) -> Result<StakingInfo, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;
        let staking = engine.staking();
        let vs = staking.active_validators();

        let validators: Vec<ValidatorStakeInfo> = vs
            .validators
            .iter()
            .filter(|v| {
                if let Some(ref hex) = pubkey_hex {
                    hex::encode(v.pubkey) == *hex
                } else {
                    true
                }
            })
            .map(|v| ValidatorStakeInfo {
                pubkey: hex::encode(v.pubkey),
                address: hex::encode(v.address),
                stake: v.stake.to_string(),
                active: v.active,
            })
            .collect();

        Ok(StakingInfo {
            validators,
            total_staked: staking.total_staked().to_string(),
            min_stake: staking.min_stake().to_string(),
            bonding_period: staking.bonding_period(),
        })
    }

    async fn get_state_root(&self) -> Result<String, ErrorObjectOwned> {
        let mut sm = self.state_manager.write().await;
        let root = sm.state_root();
        Ok(hex::encode(root))
    }

    async fn get_state_proof(
        &self,
        address_hex: String,
        token_id_hex: Option<String>,
    ) -> Result<StateProofInfo, ErrorObjectOwned> {
        let address_bytes = hex::decode(address_hex.trim_start_matches("0x")).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid address: {}", e), None::<()>)
        })?;
        if address_bytes.len() != 20 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "address must be 20 bytes",
                None::<()>,
            ));
        }
        let mut address = [0u8; 20];
        address.copy_from_slice(&address_bytes);

        let token_id = if let Some(ref hex_str) = token_id_hex {
            let bytes = hex::decode(hex_str.trim_start_matches("0x")).map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("invalid token_id: {}", e), None::<()>)
            })?;
            if bytes.len() != 32 {
                return Err(ErrorObjectOwned::owned(
                    -32602,
                    "token_id must be 32 bytes",
                    None::<()>,
                ));
            }
            let mut id = [0u8; 32];
            id.copy_from_slice(&bytes);
            id
        } else {
            NATIVE_TOKEN_ID
        };

        let mut sm = self.state_manager.write().await;
        let balance = sm.get_balance(&address, &token_id);
        let proof = sm.state_proof(&address, &token_id);
        let root = sm.state_root();

        Ok(StateProofInfo {
            address: format!("0x{}", hex::encode(address)),
            token_id: hex::encode(token_id),
            balance: balance.to_string(),
            state_root: hex::encode(root),
            proof: proof.siblings.iter().map(hex::encode).collect(),
        })
    }

    async fn get_block_transactions(
        &self,
        height: u64,
    ) -> Result<Option<BlockTransactionsInfo>, ErrorObjectOwned> {
        let sm = self.state_manager.read().await;
        let block = match sm.get_block_by_height(height) {
            Some(b) => b,
            None => return Ok(None),
        };

        let transfers = block
            .transfers
            .iter()
            .map(|bt| {
                let memo = bt
                    .memo
                    .as_ref()
                    .and_then(|m| String::from_utf8(m.clone()).ok());
                BlockTransferInfo {
                    from: format_address(&bt.from),
                    to: format_address(&bt.to),
                    token_id: hex::encode(bt.token_id),
                    amount: bt.amount.to_string(),
                    human_readable: format_amount_for_token(bt.amount, &bt.token_id, &sm),
                    memo,
                    knot_id: hex::encode(bt.knot_id),
                    timestamp: bt.timestamp,
                }
            })
            .collect();

        let token_definitions = block
            .token_definitions
            .iter()
            .map(|td| BlockTokenDefinitionInfo {
                name: td.name.clone(),
                symbol: td.symbol.clone(),
                decimals: td.decimals,
                max_supply: td.max_supply.to_string(),
                initial_supply: td.initial_supply.to_string(),
                creator: format_address(&td.creator),
                timestamp: td.timestamp,
            })
            .collect();

        let token_mints = block
            .token_mints
            .iter()
            .map(|tm| BlockTokenMintInfo {
                token_id: hex::encode(tm.token_id),
                to: format_address(&tm.to),
                amount: tm.amount.to_string(),
                timestamp: tm.timestamp,
            })
            .collect();

        let token_burns = block
            .token_burns
            .iter()
            .map(|tb| BlockTokenBurnInfo {
                token_id: hex::encode(tb.token_id),
                burner: format_address(&tb.burner),
                amount: tb.amount.to_string(),
                timestamp: tb.timestamp,
            })
            .collect();

        let name_registrations = block
            .name_registrations
            .iter()
            .map(|nr| BlockNameRegistrationInfo {
                name: nr.name.clone(),
                owner: format_address(&nr.owner),
                fee_paid: nr.fee_paid.to_string(),
                timestamp: nr.timestamp,
            })
            .collect();

        let loom_deploys = block
            .loom_deploys
            .iter()
            .map(|ld| BlockLoomDeployInfo {
                name: ld.config.name.clone(),
                operator: hex::encode(ld.operator),
                timestamp: ld.timestamp,
            })
            .collect();

        Ok(Some(BlockTransactionsInfo {
            height: block.height,
            hash: hex::encode(block.hash),
            timestamp: block.timestamp,
            transfers,
            token_definitions,
            token_mints,
            token_burns,
            name_registrations,
            loom_deploys,
        }))
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
            name_registration_count: 0,
            transfer_count: 0,
            token_definition_count: 0,
            token_mint_count: 0,
            token_burn_count: 0,
            loom_deploy_count: 0,
            stake_operation_count: 0,
            state_root: String::new(),
        };
    }
}
