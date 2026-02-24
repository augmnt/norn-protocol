use std::sync::Arc;
use tokio::sync::RwLock;

use jsonrpsee::server::{ServerBuilder, ServerHandle};

use norn_loom::lifecycle::LoomManager;
use norn_relay::relay::RelayHandle;
use norn_types::network::NetworkId;
use norn_weave::engine::WeaveEngine;

use super::chat_store::ChatEventStore;
use super::handlers::{NornRpcImpl, NornRpcServer};
use super::types::{
    BlockInfo, ChatEvent, LoomExecutionEvent, PendingTransactionEvent, TokenEvent, TransferEvent,
};
use crate::error::NodeError;
use crate::metrics::NodeMetrics;
use crate::state_manager::StateManager;

/// Groups all broadcast channels for WebSocket subscription events.
#[derive(Clone)]
pub struct RpcBroadcasters {
    pub block_tx: tokio::sync::broadcast::Sender<BlockInfo>,
    pub transfer_tx: tokio::sync::broadcast::Sender<TransferEvent>,
    pub token_tx: tokio::sync::broadcast::Sender<TokenEvent>,
    pub loom_tx: tokio::sync::broadcast::Sender<LoomExecutionEvent>,
    pub pending_tx: tokio::sync::broadcast::Sender<PendingTransactionEvent>,
    pub chat_tx: tokio::sync::broadcast::Sender<ChatEvent>,
}

impl Default for RpcBroadcasters {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcBroadcasters {
    /// Create a new set of broadcast channels.
    pub fn new() -> Self {
        let (block_tx, _) = tokio::sync::broadcast::channel::<BlockInfo>(64);
        let (transfer_tx, _) = tokio::sync::broadcast::channel::<TransferEvent>(256);
        let (token_tx, _) = tokio::sync::broadcast::channel::<TokenEvent>(64);
        let (loom_tx, _) = tokio::sync::broadcast::channel::<LoomExecutionEvent>(64);
        let (pending_tx, _) = tokio::sync::broadcast::channel::<PendingTransactionEvent>(256);
        let (chat_tx, _) = tokio::sync::broadcast::channel::<ChatEvent>(512);
        Self {
            block_tx,
            transfer_tx,
            token_tx,
            loom_tx,
            pending_tx,
            chat_tx,
        }
    }
}

/// Start the JSON-RPC HTTP+WS server.
#[allow(clippy::too_many_arguments)]
pub async fn start_rpc_server(
    addr: &str,
    weave_engine: Arc<RwLock<WeaveEngine>>,
    state_manager: Arc<RwLock<StateManager>>,
    loom_manager: Arc<RwLock<LoomManager>>,
    metrics: Arc<NodeMetrics>,
    relay_handle: Option<RelayHandle>,
    network_id: NetworkId,
    is_validator: bool,
    api_key: Option<String>,
    last_block_production_us: Arc<std::sync::Mutex<Option<u64>>>,
) -> Result<(ServerHandle, RpcBroadcasters), NodeError> {
    let broadcasters = RpcBroadcasters::new();

    let rpc_impl = NornRpcImpl {
        weave_engine,
        state_manager,
        loom_manager,
        metrics,
        broadcasters: broadcasters.clone(),
        relay_handle,
        network_id,
        is_validator,
        faucet_tracker: std::sync::Mutex::new(std::collections::HashMap::new()),
        last_block_production_us,
        chat_store: Arc::new(std::sync::RwLock::new(ChatEventStore::new())),
    };

    let handle = if let Some(key) = api_key {
        // Build server with health + auth middleware.
        let middleware = tower::ServiceBuilder::new()
            .layer(health_middleware::HealthLayer)
            .layer(auth_middleware::AuthLayer::new(key.clone()));
        let server = ServerBuilder::default()
            .set_http_middleware(middleware)
            .build(addr)
            .await
            .map_err(|e| NodeError::RpcError {
                reason: format!("failed to build RPC server: {}", e),
            })?;
        tracing::info!(addr = %addr, "RPC server started (API key auth enabled)");
        server.start(rpc_impl.into_rpc())
    } else {
        // Build server with health middleware (open access).
        let middleware = tower::ServiceBuilder::new().layer(health_middleware::HealthLayer);
        let server = ServerBuilder::default()
            .set_http_middleware(middleware)
            .build(addr)
            .await
            .map_err(|e| NodeError::RpcError {
                reason: format!("failed to build RPC server: {}", e),
            })?;
        tracing::info!(addr = %addr, "RPC server started");
        server.start(rpc_impl.into_rpc())
    };

    Ok((handle, broadcasters))
}

/// Tower middleware that intercepts `GET /health` and returns 200 OK
/// before the request reaches jsonrpsee (which only handles POST).
mod health_middleware {
    use http::{Request, Response, StatusCode};
    use http_body_util::BodyExt;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tower::{Layer, Service};

    #[derive(Clone, Copy)]
    pub struct HealthLayer;

    impl<S> Layer<S> for HealthLayer {
        type Service = HealthService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            HealthService { inner }
        }
    }

    #[derive(Clone)]
    pub struct HealthService<S> {
        inner: S,
    }

    impl<S, B> Service<Request<B>> for HealthService<S>
    where
        S: Service<Request<jsonrpsee::server::HttpBody>> + Clone + Send + 'static,
        S::Response: From<Response<jsonrpsee::server::HttpBody>>,
        S::Future: Send,
        S::Error: Send,
        B: http_body::Body + Send + 'static,
        B::Data: Send,
        B::Error: std::fmt::Display,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, req: Request<B>) -> Self::Future {
            let is_health = req.method() == http::Method::GET && req.uri().path() == "/health";

            if is_health {
                Box::pin(async move {
                    let body = jsonrpsee::server::HttpBody::from(r#"{"status":"ok"}"#);
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(body)
                        .expect("valid response");
                    Ok(response.into())
                })
            } else {
                let mut inner = self.inner.clone();
                Box::pin(async move {
                    let (parts, body) = req.into_parts();
                    let collected = body
                        .collect()
                        .await
                        .map(|c| c.to_bytes())
                        .unwrap_or_default();
                    let new_body = jsonrpsee::server::HttpBody::from(collected.to_vec());
                    inner.call(Request::from_parts(parts, new_body)).await
                })
            }
        }
    }
}

/// Tower middleware for API key authentication on RPC mutation methods.
/// Read-only methods are whitelisted and accessible without authentication.
mod auth_middleware {
    use http::header::AUTHORIZATION;
    use http::{Request, Response, StatusCode};
    use http_body_util::BodyExt;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tower::{Layer, Service};

    /// Read-only RPC methods that don't require authentication.
    const READ_ONLY_METHODS: &[&str] = &[
        "norn_getBalance",
        "norn_getBlock",
        "norn_getLatestBlock",
        "norn_getWeaveState",
        "norn_getThread",
        "norn_getThreadState",
        "norn_health",
        "norn_getValidatorSet",
        "norn_getFeeEstimate",
        "norn_getCommitmentProof",
        "norn_getTransactionHistory",
        "norn_getRecentTransfers",
        "norn_resolveName",
        "norn_listNames",
        "norn_reverseName",
        "norn_getNameRecords",
        "norn_getMetrics",
        "norn_getTokenInfo",
        "norn_getTokenBySymbol",
        "norn_listTokens",
        "norn_getLoomInfo",
        "norn_listLooms",
        "norn_queryLoom",
        "norn_getStakingInfo",
        "norn_getValidatorRewards",
        "norn_getStateRoot",
        "norn_getStateProof",
        "norn_getBlockTransactions",
        "norn_getTransaction",
        // WebSocket subscriptions are read-only.
        "norn_subscribeNewBlocks",
        "norn_unsubscribeNewBlocks",
        "norn_subscribeTransfers",
        "norn_unsubscribeTransfers",
        "norn_subscribeTokenEvents",
        "norn_unsubscribeTokenEvents",
        "norn_subscribeLoomEvents",
        "norn_unsubscribeLoomEvents",
        "norn_subscribePendingTransactions",
        "norn_unsubscribePendingTransactions",
        // Chat relay (read-only).
        "norn_getChatHistory",
        "norn_subscribeChatEvents",
        "norn_unsubscribeChatEvents",
    ];

    /// Tower layer that wraps services with API key authentication.
    #[derive(Clone)]
    pub struct AuthLayer {
        api_key: String,
    }

    impl AuthLayer {
        pub fn new(api_key: String) -> Self {
            Self { api_key }
        }
    }

    impl<S> Layer<S> for AuthLayer {
        type Service = AuthService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            AuthService {
                inner,
                api_key: self.api_key.clone(),
            }
        }
    }

    /// Tower service that checks the Authorization header on mutation RPC methods.
    /// Read-only methods are allowed without authentication.
    #[derive(Clone)]
    pub struct AuthService<S> {
        inner: S,
        api_key: String,
    }

    /// Extract all JSON-RPC method names from a request body.
    /// Handles both single requests and batch requests (JSON arrays).
    fn extract_method_names(body: &[u8]) -> Vec<String> {
        let v: serde_json::Value = match serde_json::from_slice(body) {
            Ok(v) => v,
            Err(_) => return vec![],
        };
        match &v {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|item| item.get("method")?.as_str().map(|s| s.to_string()))
                .collect(),
            serde_json::Value::Object(_) => v
                .get("method")
                .and_then(|m| m.as_str())
                .map(|s| vec![s.to_string()])
                .unwrap_or_default(),
            _ => vec![],
        }
    }

    fn is_read_only(method: &str) -> bool {
        READ_ONLY_METHODS.contains(&method)
    }

    /// Constant-time string comparison to prevent timing side-channel attacks.
    fn constant_time_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut result = 0u8;
        for (x, y) in a.bytes().zip(b.bytes()) {
            result |= x ^ y;
        }
        result == 0
    }

    impl<S, B> Service<Request<B>> for AuthService<S>
    where
        S: Service<Request<jsonrpsee::server::HttpBody>> + Clone + Send + 'static,
        S::Response: From<Response<jsonrpsee::server::HttpBody>>,
        S::Future: Send,
        S::Error: Send,
        B: http_body::Body + Send + 'static,
        B::Data: Send,
        B::Error: std::fmt::Display,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, req: Request<B>) -> Self::Future {
            let mut inner = self.inner.clone();
            let api_key = self.api_key.clone();

            Box::pin(async move {
                // Non-POST requests pass through (e.g., WebSocket upgrades).
                if req.method() != http::Method::POST {
                    let (parts, body) = req.into_parts();
                    let collected = body
                        .collect()
                        .await
                        .map(|c| c.to_bytes())
                        .unwrap_or_default();
                    let new_body = jsonrpsee::server::HttpBody::from(collected.to_vec());
                    return inner.call(Request::from_parts(parts, new_body)).await;
                }

                // Check auth header upfront.
                let authorized = req
                    .headers()
                    .get(AUTHORIZATION)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.strip_prefix("Bearer "))
                    .map(|token| constant_time_eq(token, &api_key))
                    .unwrap_or(false);

                // If authorized, pass through immediately.
                if authorized {
                    let (parts, body) = req.into_parts();
                    let collected = body
                        .collect()
                        .await
                        .map(|c| c.to_bytes())
                        .unwrap_or_default();
                    let new_body = jsonrpsee::server::HttpBody::from(collected.to_vec());
                    return inner.call(Request::from_parts(parts, new_body)).await;
                }

                // Not authorized — collect body and check if it's a read-only method.
                let (parts, body) = req.into_parts();
                let collected = body
                    .collect()
                    .await
                    .map(|c| c.to_bytes())
                    .unwrap_or_default();

                let method_names = extract_method_names(&collected);
                let is_read =
                    !method_names.is_empty() && method_names.iter().all(|m| is_read_only(m));

                if is_read {
                    // Read-only method — allow without auth.
                    let new_body = jsonrpsee::server::HttpBody::from(collected.to_vec());
                    inner.call(Request::from_parts(parts, new_body)).await
                } else {
                    // Mutation method — reject without auth.
                    let body = jsonrpsee::server::HttpBody::from(
                        r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"unauthorized: invalid or missing API key"},"id":null}"#,
                    );
                    let response = Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header("Content-Type", "application/json")
                        .body(body)
                        .expect("valid response");
                    Ok(response.into())
                }
            })
        }
    }
}
