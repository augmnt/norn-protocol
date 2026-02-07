use indicatif::{ProgressBar, ProgressStyle};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::rpc_params;

use crate::rpc::types::{BlockInfo, SubmitResult, TransactionHistoryEntry, WeaveStateInfo};

use super::error::WalletError;

/// JSON-RPC client for the Norn node.
pub struct RpcClient {
    client: HttpClient,
}

impl RpcClient {
    /// Create a new RPC client.
    pub fn new(url: &str) -> Result<Self, WalletError> {
        let client = HttpClientBuilder::default()
            .build(url)
            .map_err(|e| WalletError::RpcError(format!("failed to connect: {}", e)))?;
        Ok(Self { client })
    }

    /// Create a spinner for an RPC operation.
    fn spinner(msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("  {spinner} {msg}")
                .expect("valid template"),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb
    }

    /// Wrap an RPC call with a better connection error message.
    fn map_rpc_error(e: &jsonrpsee::core::ClientError) -> WalletError {
        let msg = e.to_string();
        if msg.contains("connection") || msg.contains("Connection") || msg.contains("refused") {
            WalletError::RpcError(
                "Could not connect to node.\nHint: Start a node with `norn-node run --dev`"
                    .to_string(),
            )
        } else {
            WalletError::RpcError(msg)
        }
    }

    /// Get a block by height.
    pub async fn get_block(&self, height: u64) -> Result<Option<BlockInfo>, WalletError> {
        let pb = Self::spinner("Fetching block...");
        let result: Option<BlockInfo> = self
            .client
            .request("norn_getBlock", rpc_params![height])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get the latest block.
    pub async fn get_latest_block(&self) -> Result<Option<BlockInfo>, WalletError> {
        let pb = Self::spinner("Fetching latest block...");
        let result: Option<BlockInfo> = self
            .client
            .request("norn_getLatestBlock", rpc_params![])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get the current weave state.
    pub async fn get_weave_state(&self) -> Result<Option<WeaveStateInfo>, WalletError> {
        let pb = Self::spinner("Fetching weave state...");
        let result: Option<WeaveStateInfo> = self
            .client
            .request("norn_getWeaveState", rpc_params![])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Submit a commitment (hex-encoded borsh bytes).
    pub async fn submit_commitment(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Submitting commitment...");
        let result: SubmitResult = self
            .client
            .request("norn_submitCommitment", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Submit a registration (hex-encoded borsh bytes).
    pub async fn submit_registration(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Submitting registration...");
        let result: SubmitResult = self
            .client
            .request("norn_submitRegistration", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get thread info.
    pub async fn get_thread(
        &self,
        thread_id: &str,
    ) -> Result<Option<crate::rpc::types::ThreadInfo>, WalletError> {
        let pb = Self::spinner("Fetching thread info...");
        let result: Option<crate::rpc::types::ThreadInfo> = self
            .client
            .request("norn_getThread", rpc_params![thread_id])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get balance for an address.
    pub async fn get_balance(&self, address: &str, token_id: &str) -> Result<String, WalletError> {
        let pb = Self::spinner("Fetching balance...");
        let result: String = self
            .client
            .request("norn_getBalance", rpc_params![address, token_id])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Request tokens from faucet.
    pub async fn faucet(&self, address: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Requesting tokens from faucet...");
        let result: SubmitResult = self
            .client
            .request("norn_faucet", rpc_params![address])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get thread state info.
    pub async fn get_thread_state(
        &self,
        thread_id: &str,
    ) -> Result<Option<crate::rpc::types::ThreadStateInfo>, WalletError> {
        let pb = Self::spinner("Fetching thread state...");
        let result: Option<crate::rpc::types::ThreadStateInfo> = self
            .client
            .request("norn_getThreadState", rpc_params![thread_id])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Submit a knot (hex-encoded borsh bytes).
    pub async fn submit_knot(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Submitting knot...");
        let result: SubmitResult = self
            .client
            .request("norn_submitKnot", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get transaction history for an address.
    pub async fn get_transaction_history(
        &self,
        address: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TransactionHistoryEntry>, WalletError> {
        let pb = Self::spinner("Fetching transaction history...");
        let result: Vec<TransactionHistoryEntry> = self
            .client
            .request(
                "norn_getTransactionHistory",
                rpc_params![address, limit, offset],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }
}
