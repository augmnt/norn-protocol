use std::sync::Arc;
use tokio::sync::RwLock;

use jsonrpsee::server::{ServerBuilder, ServerHandle};

use norn_weave::engine::WeaveEngine;

use super::handlers::{NornRpcImpl, NornRpcServer};
use super::types::BlockInfo;
use crate::error::NodeError;
use crate::metrics::NodeMetrics;

/// Start the JSON-RPC HTTP+WS server.
pub async fn start_rpc_server(
    addr: &str,
    weave_engine: Arc<RwLock<WeaveEngine>>,
    metrics: Arc<NodeMetrics>,
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
        metrics,
        block_tx: block_tx.clone(),
    };

    let handle = server.start(rpc_impl.into_rpc());

    tracing::info!(addr = %addr, "RPC server started");

    Ok((handle, block_tx))
}
