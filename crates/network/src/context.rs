//! Context for the network.

use crypto::ed25519;
use metrics::Metrics;
use score::runtime::Grandpa;
use tokio::sync::mpsc;

use crate::Event;

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

    /// Get the event sender of the network.
    fn tx(&self) -> mpsc::UnboundedSender<Event>;
}
