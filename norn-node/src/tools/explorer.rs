use std::sync::Arc;
use tokio::sync::RwLock;

use norn_weave::engine::WeaveEngine;

use crate::rpc::types::{BlockInfo, WeaveStateInfo};

/// Backend for a block explorer, providing query methods over weave state.
pub struct ExplorerBackend {
    weave_engine: Arc<RwLock<WeaveEngine>>,
}

impl ExplorerBackend {
    /// Create a new explorer backend.
    pub fn new(weave_engine: Arc<RwLock<WeaveEngine>>) -> Self {
        Self { weave_engine }
    }

    /// Get a range of blocks starting from `start` for `count` blocks.
    ///
    /// In a full implementation, this would query storage for historical blocks.
    /// For now, it returns information about the latest block if it falls in range.
    pub async fn get_blocks(&self, start: u64, count: u64) -> Vec<BlockInfo> {
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();

        let mut blocks = Vec::new();
        let end = start + count;
        if state.height >= start && state.height < end {
            blocks.push(BlockInfo {
                height: state.height,
                hash: hex::encode(state.latest_hash),
                prev_hash: String::new(),
                timestamp: 0,
                proposer: String::new(),
                commitment_count: 0,
                registration_count: 0,
                anchor_count: 0,
                fraud_proof_count: 0,
            });
        }
        blocks
    }

    /// Search for a thread by its ID (hex-encoded).
    ///
    /// Placeholder: returns None for now as we don't have a thread index yet.
    pub async fn search_thread(&self, _thread_id: &str) -> Option<String> {
        // In a full implementation, this would look up the thread in a thread index.
        None
    }

    /// Get detailed information about a specific block by height.
    pub async fn get_block_detail(&self, height: u64) -> Option<BlockInfo> {
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();

        if height == state.height {
            Some(BlockInfo {
                height: state.height,
                hash: hex::encode(state.latest_hash),
                prev_hash: String::new(),
                timestamp: 0,
                proposer: String::new(),
                commitment_count: 0,
                registration_count: 0,
                anchor_count: 0,
                fraud_proof_count: 0,
            })
        } else {
            None
        }
    }

    /// Get the current weave state summary.
    pub async fn get_weave_state(&self) -> WeaveStateInfo {
        let engine = self.weave_engine.read().await;
        let state = engine.weave_state();

        WeaveStateInfo {
            height: state.height,
            latest_hash: hex::encode(state.latest_hash),
            threads_root: hex::encode(state.threads_root),
            thread_count: state.thread_count,
            base_fee: state.fee_state.base_fee.to_string(),
            fee_multiplier: state.fee_state.fee_multiplier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norn_crypto::address::pubkey_to_address;
    use norn_crypto::keys::Keypair;
    use norn_types::weave::{FeeState, Validator, ValidatorSet, WeaveState};

    fn make_test_engine() -> Arc<RwLock<WeaveEngine>> {
        let kp = Keypair::generate();
        let vs = ValidatorSet {
            validators: vec![Validator {
                pubkey: kp.public_key(),
                address: pubkey_to_address(&kp.public_key()),
                stake: 1000,
                active: true,
            }],
            total_stake: 1000,
            epoch: 0,
        };
        let state = WeaveState {
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
        Arc::new(RwLock::new(WeaveEngine::new(kp, vs, state)))
    }

    #[tokio::test]
    async fn test_explorer_get_blocks() {
        let engine = make_test_engine();
        let explorer = ExplorerBackend::new(engine);
        let blocks = explorer.get_blocks(0, 10).await;
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].height, 0);
    }

    #[tokio::test]
    async fn test_explorer_get_block_detail() {
        let engine = make_test_engine();
        let explorer = ExplorerBackend::new(engine);
        let block = explorer.get_block_detail(0).await;
        assert!(block.is_some());
        assert_eq!(block.unwrap().height, 0);
    }

    #[tokio::test]
    async fn test_explorer_get_block_detail_not_found() {
        let engine = make_test_engine();
        let explorer = ExplorerBackend::new(engine);
        let block = explorer.get_block_detail(999).await;
        assert!(block.is_none());
    }

    #[tokio::test]
    async fn test_explorer_search_thread() {
        let engine = make_test_engine();
        let explorer = ExplorerBackend::new(engine);
        let result = explorer.search_thread("deadbeef").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_explorer_get_weave_state() {
        let engine = make_test_engine();
        let explorer = ExplorerBackend::new(engine);
        let state = explorer.get_weave_state().await;
        assert_eq!(state.height, 0);
        assert_eq!(state.thread_count, 0);
        assert_eq!(state.fee_multiplier, 1000);
    }
}
