//! Metrics implementation of Spacejam.

use anyhow::Result;
use network::Connection;
pub use network::Peer;
use prometheus::{encoding::text, registry::Registry};
use std::sync::Arc;

mod network;

/// Metrics implementation of Spacejam.
///
/// TODO: allow to disable / enable metrics from config.
#[derive(Clone)]
pub struct Metrics {
    /// Registry.
    registry: Arc<Registry>,

    /// Connections.
    pub conn: Connection,
}

impl Metrics {
    /// Create a new metrics instance.
    pub fn new(peer: &str) -> Self {
        let mut registry = Registry::with_prefix(format!("spacejam::{peer}"));

        // network metrics
        let conn = Connection::default();
        conn.register(&mut registry);

        // wrap register in Arc
        Self {
            registry: Arc::new(registry),
            conn,
        }
    }

    /// Get the metrics as a string.
    pub fn metrics(&self) -> Result<String> {
        let mut buffer = String::new();
        text::encode(&mut buffer, self.registry.as_ref())?;
        Ok(buffer)
    }
}

/// A trait for metrics.
pub trait Metric {
    /// Register the metric.
    fn register(&self, registry: &mut Registry);
}
