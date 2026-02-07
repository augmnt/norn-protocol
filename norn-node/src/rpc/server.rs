use std::sync::Arc;
use tokio::sync::RwLock;

use jsonrpsee::server::{ServerBuilder, ServerHandle};

use norn_relay::relay::RelayHandle;
use norn_weave::engine::WeaveEngine;

use super::handlers::{NornRpcImpl, NornRpcServer};
use super::types::BlockInfo;
use crate::error::NodeError;
use crate::metrics::NodeMetrics;
use crate::state_manager::StateManager;

/// Start the JSON-RPC HTTP+WS server.
pub async fn start_rpc_server(
    addr: &str,
    weave_engine: Arc<RwLock<WeaveEngine>>,
    state_manager: Arc<RwLock<StateManager>>,
    metrics: Arc<NodeMetrics>,
    relay_handle: Option<RelayHandle>,
) -> Result<(ServerHandle, tokio::sync::broadcast::Sender<BlockInfo>), NodeError> {
    let server = ServerBuilder::default()
        .build(addr)
        .await
        .map_err(|e| NodeError::RpcError {
            reason: format!("failed to build RPC server: {}", e),
        })?;

    let (block_tx, _) = tokio::sync::broadcast::channel::<BlockInfo>(64);

    let rpc_impl = NornRpcImpl {
        weave_engine,
        state_manager,
        metrics,
        block_tx: block_tx.clone(),
        relay_handle,
    };

    let handle = server.start(rpc_impl.into_rpc());

    tracing::info!(addr = %addr, "RPC server started");

    Ok((handle, block_tx))
}
