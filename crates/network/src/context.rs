//! Context for the network.

use crypto::ed25519;
use metrics::Metrics;

/// Context for the network.
pub trait Context {
    /// Get the metrics of the node.
    fn metrics(&self) -> &Metrics;

    /// Get the keypair of the p2p network.
    fn keypair(&self) -> Option<ed25519::KeyPair> {
        None
    }
}

impl Context for Metrics {
    fn metrics(&self) -> &Metrics {
        self
    }
}
