//! Context for SpaceJam

use crypto::ed25519;
use metrics::Metrics;
use network::Action;
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
    pub tx: mpsc::UnboundedSender<Action>,
}

/// Create a new context
///
/// TODO: longest chain selection.
impl<S: Storage, V: Validator> Context<S, V> {
    /// Create a new context
    pub fn new(validator: V, db: S, tx: mpsc::UnboundedSender<Action>) -> Self {
        // TODO: use base32.
        let peer_id = hex::encode(validator.ed25519_public_key());
        Self {
            runtime: Runtime::new(validator, db),
            metrics: Metrics::new(&peer_id),
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

    // TODO: longest chain selection.
    fn import_block(&self, block: Vec<u8>) -> anyhow::Result<()> {
        self.runtime.import(block)
    }
}
