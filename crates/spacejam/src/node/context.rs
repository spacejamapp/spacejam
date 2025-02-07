//! Context for SpaceJam

use metrics::Metrics;
use network::{ed25519, PeerId};
use score::{state::Storage, validator::Validator};

/// The context for SpaceJam
///
/// TODO: maybe move this to the core library...
pub struct Context<S: Storage, V: Validator> {
    /// The validator of SpaceJam
    pub validator: V,

    /// The storage of SpaceJam
    pub db: S,

    /// The metrics of SpaceJam
    pub metrics: Metrics,
}

impl<S: Storage, V: Validator> Context<S, V> {
    /// Create a new context
    pub fn new(validator: V, db: S) -> Self {
        let peer_id = validator
            .ed25519()
            .map(|kp| PeerId::from_bytes(kp.verifying.as_bytes()).ok())
            .flatten()
            .map(|peer_id| peer_id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            validator,
            db,
            metrics: Metrics::new(&peer_id),
        }
    }
}

impl<S: Storage, V: Validator> network::Context for Context<S, V> {
    fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    fn keypair(&self) -> Option<ed25519::Keypair> {
        let kp = self.validator.ed25519()?;
        let sk = ed25519::SecretKey::try_from_bytes(kp.signing.to_bytes()).ok()?;
        Some(ed25519::Keypair::from(sk))
    }

    fn import_block(&self, block: Vec<u8>) -> anyhow::Result<()> {
        sync::transit(&codec::decode(&block)?, &self.db, &self.validator)
    }
}
