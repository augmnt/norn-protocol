use indicatif::{ProgressBar, ProgressStyle};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::rpc_params;

use crate::rpc::types::{
    BlockInfo, ExecutionResult, FeeEstimateInfo, HealthInfo, LoomInfo, NameInfo, NameResolution,
    QueryResult, StakingInfo, SubmitResult, TokenInfo, TransactionHistoryEntry,
    ValidatorRewardsInfo, ValidatorSetInfo, WeaveStateInfo,
};

use super::error::WalletError;

/// Default RPC request timeout in seconds.
const DEFAULT_RPC_TIMEOUT_SECS: u64 = 10;

/// JSON-RPC client for the Norn node.
pub struct RpcClient {
    client: HttpClient,
}

impl RpcClient {
    /// Create a new RPC client.
    pub fn new(url: &str) -> Result<Self, WalletError> {
        let client = HttpClientBuilder::default()
            .request_timeout(std::time::Duration::from_secs(DEFAULT_RPC_TIMEOUT_SECS))
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
        if msg.contains("connection")
            || msg.contains("Connection")
            || msg.contains("refused")
            || msg.contains("SendRequest")
            || msg.contains("send request")
        {
            WalletError::RpcError(
                "Could not connect to node.\nHint: Start a node with `norn run --dev`".to_string(),
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

    /// Register a name for an address.
    pub async fn register_name(
        &self,
        name: &str,
        owner_hex: &str,
        knot_hex: &str,
    ) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Registering name...");
        let result: SubmitResult = self
            .client
            .request("norn_registerName", rpc_params![name, owner_hex, knot_hex])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Resolve a name to its owner.
    pub async fn resolve_name(&self, name: &str) -> Result<Option<NameResolution>, WalletError> {
        let pb = Self::spinner("Resolving name...");
        let result: Option<NameResolution> = self
            .client
            .request("norn_resolveName", rpc_params![name])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// List names owned by an address.
    pub async fn list_names(&self, address_hex: &str) -> Result<Vec<NameInfo>, WalletError> {
        let pb = Self::spinner("Fetching names...");
        let result: Vec<NameInfo> = self
            .client
            .request("norn_listNames", rpc_params![address_hex])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get node health info.
    pub async fn health(&self) -> Result<HealthInfo, WalletError> {
        let pb = Self::spinner("Checking node health...");
        let result: HealthInfo = self
            .client
            .request("norn_health", rpc_params![])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get the current validator set.
    pub async fn get_validator_set(&self) -> Result<ValidatorSetInfo, WalletError> {
        let pb = Self::spinner("Fetching validator set...");
        let result: ValidatorSetInfo = self
            .client
            .request("norn_getValidatorSet", rpc_params![])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get fee estimate.
    pub async fn get_fee_estimate(&self) -> Result<FeeEstimateInfo, WalletError> {
        let pb = Self::spinner("Fetching fee estimate...");
        let result: FeeEstimateInfo = self
            .client
            .request("norn_getFeeEstimate", rpc_params![])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Create a new token (hex-encoded borsh TokenDefinition).
    pub async fn create_token(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Creating token...");
        let result: SubmitResult = self
            .client
            .request("norn_createToken", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Mint tokens (hex-encoded borsh TokenMint).
    pub async fn mint_token(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Minting tokens...");
        let result: SubmitResult = self
            .client
            .request("norn_mintToken", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Burn tokens (hex-encoded borsh TokenBurn).
    pub async fn burn_token(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Burning tokens...");
        let result: SubmitResult = self
            .client
            .request("norn_burnToken", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get token info by token ID (hex).
    pub async fn get_token_info(
        &self,
        token_id_hex: &str,
    ) -> Result<Option<TokenInfo>, WalletError> {
        let pb = Self::spinner("Fetching token info...");
        let result: Option<TokenInfo> = self
            .client
            .request("norn_getTokenInfo", rpc_params![token_id_hex])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get token info by symbol.
    pub async fn get_token_by_symbol(
        &self,
        symbol: &str,
    ) -> Result<Option<TokenInfo>, WalletError> {
        let pb = Self::spinner("Looking up token...");
        let result: Option<TokenInfo> = self
            .client
            .request("norn_getTokenBySymbol", rpc_params![symbol])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// List all tokens with pagination.
    pub async fn list_tokens(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TokenInfo>, WalletError> {
        let pb = Self::spinner("Fetching tokens...");
        let result: Vec<TokenInfo> = self
            .client
            .request("norn_listTokens", rpc_params![limit, offset])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Deploy a loom (smart contract).
    pub async fn deploy_loom(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Deploying loom...");
        let result: SubmitResult = self
            .client
            .request("norn_deployLoom", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get loom info by ID.
    pub async fn get_loom_info(&self, loom_id_hex: &str) -> Result<Option<LoomInfo>, WalletError> {
        let pb = Self::spinner("Fetching loom info...");
        let result: Option<LoomInfo> = self
            .client
            .request("norn_getLoomInfo", rpc_params![loom_id_hex])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// List all deployed looms with pagination.
    pub async fn list_looms(&self, limit: u64, offset: u64) -> Result<Vec<LoomInfo>, WalletError> {
        let pb = Self::spinner("Fetching looms...");
        let result: Vec<LoomInfo> = self
            .client
            .request("norn_listLooms", rpc_params![limit, offset])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Upload bytecode to a deployed loom with operator authentication.
    pub async fn upload_loom_bytecode(
        &self,
        loom_id_hex: &str,
        bytecode_hex: &str,
        init_msg_hex: Option<&str>,
        operator_signature_hex: &str,
        operator_pubkey_hex: &str,
    ) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Uploading bytecode...");
        let result: SubmitResult = self
            .client
            .request(
                "norn_uploadLoomBytecode",
                rpc_params![
                    loom_id_hex,
                    bytecode_hex,
                    init_msg_hex,
                    operator_signature_hex,
                    operator_pubkey_hex
                ],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Execute a loom contract with sender authentication.
    pub async fn execute_loom(
        &self,
        loom_id_hex: &str,
        input_hex: &str,
        sender_hex: &str,
        signature_hex: &str,
        pubkey_hex: &str,
    ) -> Result<ExecutionResult, WalletError> {
        let pb = Self::spinner("Executing loom...");
        let result: ExecutionResult = self
            .client
            .request(
                "norn_executeLoom",
                rpc_params![
                    loom_id_hex,
                    input_hex,
                    sender_hex,
                    signature_hex,
                    pubkey_hex
                ],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Query a loom contract (read-only).
    pub async fn query_loom(
        &self,
        loom_id_hex: &str,
        input_hex: &str,
    ) -> Result<QueryResult, WalletError> {
        let pb = Self::spinner("Querying loom...");
        let result: QueryResult = self
            .client
            .request("norn_queryLoom", rpc_params![loom_id_hex, input_hex])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Join a loom as a participant.
    pub async fn join_loom(
        &self,
        loom_id_hex: &str,
        participant_hex: &str,
        pubkey_hex: &str,
        signature_hex: &str,
    ) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Joining loom...");
        let result: SubmitResult = self
            .client
            .request(
                "norn_joinLoom",
                rpc_params![loom_id_hex, participant_hex, pubkey_hex, signature_hex],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Leave a loom.
    pub async fn leave_loom(
        &self,
        loom_id_hex: &str,
        participant_hex: &str,
        signature_hex: &str,
        pubkey_hex: &str,
    ) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Leaving loom...");
        let result: SubmitResult = self
            .client
            .request(
                "norn_leaveLoom",
                rpc_params![loom_id_hex, participant_hex, signature_hex, pubkey_hex],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    pub async fn submit_stake(&self, hex_data: &str) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Submitting stake operation...");
        let result: SubmitResult = self
            .client
            .request("norn_stake", rpc_params![hex_data])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    pub async fn get_staking_info(
        &self,
        pubkey_hex: Option<&str>,
    ) -> Result<StakingInfo, WalletError> {
        let pb = Self::spinner("Fetching staking info...");
        let result: StakingInfo = self
            .client
            .request(
                "norn_getStakingInfo",
                rpc_params![pubkey_hex.map(|s| s.to_string())],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Transfer a name to a new owner.
    pub async fn transfer_name(
        &self,
        name: &str,
        from_hex: &str,
        transfer_hex: &str,
    ) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Transferring name...");
        let result: SubmitResult = self
            .client
            .request(
                "norn_transferName",
                rpc_params![name, from_hex, transfer_hex],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Reverse-resolve an address to its primary name.
    pub async fn reverse_name(&self, address_hex: &str) -> Result<Option<String>, WalletError> {
        let pb = Self::spinner("Looking up name...");
        let result: Option<String> = self
            .client
            .request("norn_reverseName", rpc_params![address_hex])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Set a record on a name.
    pub async fn set_name_record(
        &self,
        name: &str,
        key: &str,
        value: &str,
        owner_hex: &str,
        knot_hex: &str,
    ) -> Result<SubmitResult, WalletError> {
        let pb = Self::spinner("Setting name record...");
        let result: SubmitResult = self
            .client
            .request(
                "norn_setNameRecord",
                rpc_params![name, key, value, owner_hex, knot_hex],
            )
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get all records for a name.
    pub async fn get_name_records(
        &self,
        name: &str,
    ) -> Result<std::collections::HashMap<String, String>, WalletError> {
        let pb = Self::spinner("Fetching name records...");
        let result: std::collections::HashMap<String, String> = self
            .client
            .request("norn_getNameRecords", rpc_params![name])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }

    /// Get validator rewards info.
    pub async fn get_validator_rewards(&self) -> Result<ValidatorRewardsInfo, WalletError> {
        let pb = Self::spinner("Fetching validator rewards...");
        let result: ValidatorRewardsInfo = self
            .client
            .request("norn_getValidatorRewards", rpc_params![])
            .await
            .map_err(|e| Self::map_rpc_error(&e))?;
        pb.finish_and_clear();
        Ok(result)
    }
}
