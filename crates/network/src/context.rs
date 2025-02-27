//! Context for the network.

use crate::peer::Manager;
use crypto::ed25519;
use metrics::Metrics;
use score::runtime::Grandpa;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Context for the network.
#[allow(async_fn_in_trait)]
pub trait Context {
    /// Get the keypair of the p2p network.
    fn keypair(&self) -> Option<ed25519::KeyPair> {
        None
    }

    /// Get the metrics of the node.
    fn metrics(&self) -> &Metrics;

    /// Announce the handshake message.
    fn grandpa(&self) -> Grandpa;

    /// Get the manager of the network.
    fn manager(&self) -> Arc<RwLock<Manager>>;
}
