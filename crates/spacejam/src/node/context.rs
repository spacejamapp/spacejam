//! Context for SpaceJam

use anyhow::Result;
use crypto::ed25519;
use metrics::Metrics;
use network::{event::action, peer::PeerId};
use score::runtime::{Runtime, Storage, Validator};
use tokio::sync::mpsc;

/// The context for SpaceJam
///
/// TODO: maybe move this to the core library...
pub struct Context<S: Storage, V: Validator> {
    /// The runtime of SpaceJam
    pub runtime: Runtime<S, V>,

    /// The metrics of SpaceJam
    pub metrics: Metrics,

    /// The event sender
    pub tx: mpsc::UnboundedSender<action::Event>,
}

/// Create a new context
///
/// TODO: longest chain selection.
impl<S: Storage, V: Validator> Context<S, V> {
    /// Create a new context
    pub fn new(validator: V, db: S, tx: mpsc::UnboundedSender<action::Event>) -> Self {
        let peer_id = PeerId::from(&validator.ed25519_public_key());
        Self {
            runtime: Runtime::new(validator, db),
            metrics: Metrics::new(peer_id.as_ref()),
            tx,
        }
    }
}

impl<S: Storage, V: Validator> network::Context for Context<S, V> {
    fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    fn keypair(&self) -> Option<ed25519::KeyPair> {
        self.runtime.validator.ed25519()
    }

    async fn up0_handshake(&self) -> Result<Vec<u8>> {
        let grandpa = self.runtime.grandpa.clone().read().await.clone();
        let mut handshake = vec![];
        handshake.extend_from_slice(grandpa.head.hash()?.as_ref());
        handshake.extend_from_slice(&grandpa.head.slot.to_le_bytes());
        handshake.extend_from_slice(&grandpa.leaves.len().to_le_bytes());
        for leaf in grandpa.leaves.iter() {
            handshake.extend_from_slice(leaf.hash()?.as_ref());
            handshake.extend_from_slice(&leaf.slot.to_le_bytes());
        }

        Ok(handshake)
    }
}
