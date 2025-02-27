//! Context for SpaceJam

use crate::node::Builder;
use crypto::ed25519;
use metrics::Metrics;
use network::{event::Event, peer::PeerId, Manager};
use score::runtime::{Runtime, Storage, Validator};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// The context for SpaceJam
///
/// TODO: maybe move this to the core library...
pub struct Context<S: Storage, V: Validator> {
    /// The runtime of SpaceJam
    pub runtime: Runtime<S, V>,

    /// The metrics of SpaceJam
    pub metrics: Metrics,

    /// The manager of the network
    pub manager: Arc<RwLock<Manager>>,

    /// The event sender
    pub tx: mpsc::UnboundedSender<Event>,
}

/// Create a new context
///
/// TODO: longest chain selection.
impl<S: Storage, V: Validator> Context<S, V> {
    /// Create a new context builder
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Create a new context
    pub fn new(validator: V, db: S, tx: mpsc::UnboundedSender<Event>) -> Self {
        let peer_id = PeerId::from(&validator.ed25519_public_key());

        // TODO: make the buffer size configurable
        let manager = Arc::new(RwLock::new(Manager::new(tx.clone())));
        Self {
            runtime: Runtime::new(validator, db),
            metrics: Metrics::new(peer_id.as_ref()),
            manager,
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

    fn grandpa(&self) -> score::runtime::Grandpa {
        self.runtime.grandpa.clone()
    }

    fn tx(&self) -> mpsc::UnboundedSender<Event> {
        self.tx.clone()
    }
}
