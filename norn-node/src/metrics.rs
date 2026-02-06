use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

/// Node-wide Prometheus metrics.
pub struct NodeMetrics {
    pub weave_height: Gauge,
    pub peer_count: Gauge,
    pub mempool_size: Gauge,
    pub blocks_produced: Counter,
    pub fraud_proofs_submitted: Counter,
    pub knots_validated: Counter,
    pub registry: Registry,
}

impl NodeMetrics {
    /// Create a new metrics registry with all node metrics registered.
    pub fn new() -> Self {
        let mut registry = Registry::default();

        let weave_height = Gauge::default();
        let peer_count = Gauge::default();
        let mempool_size = Gauge::default();
        let blocks_produced = Counter::default();
        let fraud_proofs_submitted = Counter::default();
        let knots_validated = Counter::default();

        registry.register(
            "norn_weave_height",
            "Current weave block height",
            weave_height.clone(),
        );
        registry.register(
            "norn_peer_count",
            "Number of connected peers",
            peer_count.clone(),
        );
        registry.register(
            "norn_mempool_size",
            "Number of items in the mempool",
            mempool_size.clone(),
        );
        registry.register(
            "norn_blocks_produced",
            "Total blocks produced",
            blocks_produced.clone(),
        );
        registry.register(
            "norn_fraud_proofs_submitted",
            "Total fraud proofs submitted",
            fraud_proofs_submitted.clone(),
        );
        registry.register(
            "norn_knots_validated",
            "Total knots validated",
            knots_validated.clone(),
        );

        Self {
            weave_height,
            peer_count,
            mempool_size,
            blocks_produced,
            fraud_proofs_submitted,
            knots_validated,
            registry,
        }
    }

    /// Encode all metrics in Prometheus text exposition format.
    pub fn encode(&self) -> String {
        let mut buf = String::new();
        prometheus_client::encoding::text::encode(&mut buf, &self.registry)
            .expect("encoding metrics should not fail");
        buf
    }
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = NodeMetrics::new();
        metrics.weave_height.set(42);
        metrics.peer_count.set(5);
        metrics.blocks_produced.inc();
        metrics.blocks_produced.inc();

        let encoded = metrics.encode();
        assert!(encoded.contains("norn_weave_height"));
        assert!(encoded.contains("norn_peer_count"));
        assert!(encoded.contains("norn_blocks_produced"));
    }

    #[test]
    fn test_metrics_encode_format() {
        let metrics = NodeMetrics::new();
        metrics.weave_height.set(100);
        let encoded = metrics.encode();
        // Should contain the metric name and a numeric value.
        assert!(encoded.contains("norn_weave_height"));
        assert!(encoded.contains("100"));
    }
}
