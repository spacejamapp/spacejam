//! Context for the network.

use anyhow::Result;
use crypto::ed25519;
use metrics::Metrics;

/// Context for the network.
#[allow(async_fn_in_trait)]
pub trait Context {
    /// Get the metrics of the node.
    fn metrics(&self) -> &Metrics;

    /// Get the keypair of the p2p network.
    fn keypair(&self) -> Option<ed25519::KeyPair> {
        None
    }

    /// Announce the handshake message.
    async fn up0_handshake(&self) -> Result<Vec<u8>>;
}

impl Context for Metrics {
    fn metrics(&self) -> &Metrics {
        self
    }

    async fn up0_handshake(&self) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}
