//! Context for SpaceJam

use metrics::Metrics;
use network::{ed25519, Event, PeerId};
use score::{state::Storage, validator::Validator, Block};
use tokio::sync::mpsc;

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

    /// The event sender
    pub tx: mpsc::Sender<Event>,
}

/// Create a new context
///
/// TODO: longest chain selection.
impl<S: Storage, V: Validator> Context<S, V> {
    /// Create a new context
    pub fn new(validator: V, db: S, tx: mpsc::Sender<Event>) -> Self {
        let peer_id = validator
            .ed25519()
            .and_then(|kp| PeerId::from_bytes(kp.verifying.as_bytes()).ok())
            .map(|peer_id| peer_id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            metrics: Metrics::new(&peer_id),
            validator,
            db,
            tx,
        }
    }

    /// Author a block
    pub async fn author(&self) -> anyhow::Result<()> {
        let block = self.db.recent_blocks()?;
        let Some(block) = block.and_then(|b| b.last().cloned()) else {
            return Ok(());
        };

        let block: Block = Block::builder()
            .parent(&block)?
            .seal(&self.validator, &self.db)?
            .into();

        tracing::info!(
            "subscribing block@{}: {}",
            block.header.slot,
            hex::encode(block.hash()?)
        );
        let block = codec::encode(&block)?;
        self.tx.send(Event::SubscribeBlock(block)).await?;
        Ok(())
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
