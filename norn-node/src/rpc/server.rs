use std::sync::Arc;
use tokio::sync::RwLock;

use jsonrpsee::server::{ServerBuilder, ServerHandle};

use norn_relay::relay::RelayHandle;
use norn_types::network::NetworkId;
use norn_weave::engine::WeaveEngine;

use super::handlers::{NornRpcImpl, NornRpcServer};
use super::types::BlockInfo;
use crate::error::NodeError;
use crate::metrics::NodeMetrics;
use crate::state_manager::StateManager;

/// Start the JSON-RPC HTTP+WS server.
#[allow(clippy::too_many_arguments)]
pub async fn start_rpc_server(
    addr: &str,
    weave_engine: Arc<RwLock<WeaveEngine>>,
    state_manager: Arc<RwLock<StateManager>>,
    metrics: Arc<NodeMetrics>,
    relay_handle: Option<RelayHandle>,
    network_id: NetworkId,
    is_validator: bool,
    api_key: Option<String>,
) -> Result<(ServerHandle, tokio::sync::broadcast::Sender<BlockInfo>), NodeError> {
    let (block_tx, _) = tokio::sync::broadcast::channel::<BlockInfo>(64);

    let rpc_impl = NornRpcImpl {
        weave_engine,
        state_manager,
        metrics,
        block_tx: block_tx.clone(),
        relay_handle,
        network_id,
        is_validator,
        faucet_tracker: std::sync::Mutex::new(std::collections::HashMap::new()),
    };

    let handle = if let Some(key) = api_key {
        // Build server with auth middleware.
        let middleware =
            tower::ServiceBuilder::new().layer(auth_middleware::AuthLayer::new(key.clone()));
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
        // Build server without auth middleware (open access).
        let server =
            ServerBuilder::default()
                .build(addr)
                .await
                .map_err(|e| NodeError::RpcError {
                    reason: format!("failed to build RPC server: {}", e),
                })?;
        tracing::info!(addr = %addr, "RPC server started");
        server.start(rpc_impl.into_rpc())
    };

    Ok((handle, block_tx))
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
        "norn_resolveName",
        "norn_listNames",
        "norn_getMetrics",
        "norn_getNodeInfo",
        "norn_getTokenInfo",
        "norn_getTokenBySymbol",
        "norn_listTokens",
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

    /// Extract the JSON-RPC method name from a request body (best-effort).
    /// Looks for `"method":"..."` or `"method": "..."` patterns without
    /// requiring a full JSON parse.
    fn extract_method_name(body: &[u8]) -> Option<String> {
        let text = std::str::from_utf8(body).ok()?;
        // Fast path: find "method" key and extract its string value.
        let method_idx = text.find("\"method\"")?;
        let after_key = &text[method_idx + 8..];
        // Skip whitespace and colon
        let after_colon = after_key.find(':').map(|i| &after_key[i + 1..])?;
        let trimmed = after_colon.trim_start();
        if !trimmed.starts_with('"') {
            return None;
        }
        let value_start = 1; // skip opening quote
        let value_end = trimmed[value_start..].find('"')?;
        Some(trimmed[value_start..value_start + value_end].to_string())
    }

    fn is_read_only(method: &str) -> bool {
        READ_ONLY_METHODS.contains(&method)
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
                    .map(|token| token == api_key)
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

                let method_name = extract_method_name(&collected);
                let is_read = method_name.as_deref().map(is_read_only).unwrap_or(false);

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
