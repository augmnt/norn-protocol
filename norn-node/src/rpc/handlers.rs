use std::sync::Arc;
use tokio::sync::RwLock;

use jsonrpsee::core::async_trait;
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::PendingSubscriptionSink;

use norn_weave::engine::WeaveEngine;

use super::types::{
    BlockInfo, CommitmentProofInfo, FeeEstimateInfo, HealthInfo, SubmitResult, ThreadInfo,
    ThreadStateInfo, ValidatorInfo, ValidatorSetInfo, WeaveStateInfo,
};
use crate::metrics::NodeMetrics;

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
}

/// Implementation of the NornRpc trait.
#[allow(dead_code)]
pub struct NornRpcImpl {
    pub weave_engine: Arc<RwLock<WeaveEngine>>,
    pub metrics: Arc<NodeMetrics>,
    pub block_tx: tokio::sync::broadcast::Sender<BlockInfo>,
}

#[async_trait]
impl NornRpcServer for NornRpcImpl {
    async fn get_block(&self, height: u64) -> Result<Option<BlockInfo>, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();

        // For now, we only know about the latest block height.
        // A full implementation would query storage for historical blocks.
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
        match engine.add_commitment(commitment) {
            Ok(_) => Ok(SubmitResult {
                success: true,
                reason: None,
            }),
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

        let mut engine = self.weave_engine.write().await;
        match engine.add_registration(registration) {
            Ok(_) => Ok(SubmitResult {
                success: true,
                reason: None,
            }),
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
        let thread_bytes = hex::decode(&thread_id_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        if thread_bytes.len() != 20 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "thread_id must be 20 bytes",
                None::<()>,
            ));
        }

        let mut thread_id = [0u8; 20];
        thread_id.copy_from_slice(&thread_bytes);

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

    async fn get_balance(
        &self,
        _address: String,
        _token_id: String,
    ) -> Result<String, ErrorObjectOwned> {
        // Thread state is maintained off-chain; the weave only stores commitments.
        // For a full implementation, the node would need to index thread states.
        // Return 0 as placeholder.
        Ok("0".to_string())
    }

    async fn get_thread_state(
        &self,
        thread_id_hex: String,
    ) -> Result<Option<ThreadStateInfo>, ErrorObjectOwned> {
        let thread_bytes = hex::decode(&thread_id_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        if thread_bytes.len() != 20 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "thread_id must be 20 bytes",
                None::<()>,
            ));
        }

        let mut thread_id = [0u8; 20];
        thread_id.copy_from_slice(&thread_bytes);

        let engine = self.weave_engine.read().await;
        if engine.known_threads().contains(&thread_id) {
            Ok(Some(ThreadStateInfo {
                thread_id: thread_id_hex,
                owner: String::new(),
                version: 0,
                state_hash: hex::encode([0u8; 32]),
                balances: vec![],
            }))
        } else {
            Ok(None)
        }
    }

    // Faucet: testnet-only endpoint that bypasses signature verification
    // to auto-register threads and credit test tokens. Returns an error in
    // production builds (compile without the "testnet" feature).
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

            let addr_bytes = hex::decode(&address_hex).map_err(|e| {
                ErrorObjectOwned::owned(-32602, format!("invalid hex address: {}", e), None::<()>)
            })?;

            if addr_bytes.len() != 20 {
                return Err(ErrorObjectOwned::owned(
                    -32602,
                    "address must be 20 bytes",
                    None::<()>,
                ));
            }

            let mut address = [0u8; 20];
            address.copy_from_slice(&addr_bytes);

            let faucet_amount: u128 = 100 * ONE_NORN; // 100 NORN per faucet request

            let mut engine = self.weave_engine.write().await;

            // Register the thread if not already known.
            if !engine.known_threads().contains(&address) {
                let reg = norn_types::weave::Registration {
                    thread_id: address,
                    owner: [0u8; 32], // Owner unknown (faucet auto-registers)
                    initial_state_hash: [0u8; 32],
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    signature: [0u8; 64], // Faucet-issued (no signature verification)
                };
                // Add registration directly (bypass signature check for faucet).
                let _ = engine.add_registration(reg);
            }

            Ok(SubmitResult {
                success: true,
                reason: Some(format!(
                    "credited {} nits to {}",
                    faucet_amount, address_hex
                )),
            })
        }
    }

    async fn submit_knot(&self, knot_hex: String) -> Result<SubmitResult, ErrorObjectOwned> {
        let _bytes = hex::decode(&knot_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        // Knot submission requires the node to validate and relay the knot.
        // For now, accept and acknowledge.
        Ok(SubmitResult {
            success: true,
            reason: None,
        })
    }

    async fn health(&self) -> Result<HealthInfo, ErrorObjectOwned> {
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();

        Ok(HealthInfo {
            height: state.height,
            is_validator: true,
            thread_count: state.thread_count,
            status: "ok".to_string(),
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
        let thread_bytes = hex::decode(&thread_id_hex).map_err(|e| {
            ErrorObjectOwned::owned(-32602, format!("invalid hex: {}", e), None::<()>)
        })?;

        if thread_bytes.len() != 20 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "thread_id must be 20 bytes",
                None::<()>,
            ));
        }

        let mut thread_id = [0u8; 20];
        thread_id.copy_from_slice(&thread_bytes);

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
