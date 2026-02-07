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

/// Tower middleware for API key authentication on RPC POST requests.
mod auth_middleware {
    use http::header::AUTHORIZATION;
    use http::{Request, Response, StatusCode};
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tower::{Layer, Service};

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

    /// Tower service that checks the Authorization header on POST requests.
    #[derive(Clone)]
    pub struct AuthService<S> {
        inner: S,
        api_key: String,
    }

    impl<S, B> Service<Request<B>> for AuthService<S>
    where
        S: Service<Request<B>> + Clone + Send + 'static,
        S::Response: From<Response<jsonrpsee::server::HttpBody>>,
        S::Future: Send,
        S::Error: Send,
        B: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, req: Request<B>) -> Self::Future {
            // Only check POST requests (all JSON-RPC calls are POST).
            if req.method() == http::Method::POST {
                let authorized = req
                    .headers()
                    .get(AUTHORIZATION)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.strip_prefix("Bearer "))
                    .map(|token| token == self.api_key)
                    .unwrap_or(false);

                if !authorized {
                    return Box::pin(async {
                        let body = jsonrpsee::server::HttpBody::from(
                            r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"unauthorized: invalid or missing API key"},"id":null}"#,
                        );
                        let response = Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("Content-Type", "application/json")
                            .body(body)
                            .expect("valid response");
                        Ok(response.into())
                    });
                }
            }

            let mut inner = self.inner.clone();
            Box::pin(async move { inner.call(req).await })
        }
    }
}
