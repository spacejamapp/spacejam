//! Metrics implementation of Spacejam.

use anyhow::Result;
pub use network::Peer;
use prometheus::{
    encoding::text,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};
use std::sync::Arc;

mod network;

/// Metrics implementation of Spacejam.
///
/// TODO: allow to disable / enable metrics from config.
pub struct Metrics {
    registry: Arc<Registry>,

    /// Connections.
    pub connections: Family<Peer, Gauge>,
}

impl Metrics {
    /// Create a new metrics instance.
    pub fn new(peer: &str) -> Self {
        let mut registry = Registry::with_prefix(format!("spacejam::{peer}"));
        let connections = Family::<Peer, Gauge>::default();
        registry.register(
            "conn",
            "Connection status, 1: established, 0: closed",
            connections.clone(),
        );

        Self {
            registry: Arc::new(registry),
            connections,
        }
    }

    /// Increment the established connection counter.
    pub fn establish_connection(&self, peer: String) {
        self.connections
            .get_or_create(&Peer { peer })
            .set(Peer::established());
    }

    /// Decrement the connection counter.
    pub fn close_connection(&self, peer: String) {
        self.connections
            .get_or_create(&Peer { peer })
            .set(Peer::closed());
    }

    /// Get the metrics as a string.
    pub fn metrics(&self) -> Result<String> {
        let mut buffer = String::new();
        text::encode(&mut buffer, self.registry.as_ref())?;
        Ok(buffer)
    }
}
