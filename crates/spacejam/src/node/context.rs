//! Context for SpaceJam

use metrics::Metrics;
use network::{ed25519, Event, PeerId};
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
    pub tx: mpsc::Sender<Event>,
}

/// Create a new context
///
/// TODO: longest chain selection.
impl<S: Storage, V: Validator> Context<S, V> {
    /// Create a new context
    pub fn new(validator: V, db: S, tx: mpsc::Sender<Event>) -> Self {
        let peer_id = PeerId::from_bytes(&validator.ed25519_public_key())
            .ok()
            .map(|peer_id| peer_id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

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

    fn keypair(&self) -> Option<ed25519::Keypair> {
        let kp = self.runtime.validator.ed25519()?;
        let sk = ed25519::SecretKey::try_from_bytes(kp.signing.to_bytes()).ok()?;
        Some(ed25519::Keypair::from(sk))
    }

    // TODO: longest chain selection.
    fn import_block(&self, block: Vec<u8>) -> anyhow::Result<()> {
        self.runtime.import(block)
    }
}
