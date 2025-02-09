//! Node for SpaceJam

use network::{Event, Network};
use score::runtime::{Storage, Validator};
use std::{net::SocketAddr, time::Duration};
pub use {builder::Builder, context::Context, genesis::Genesis};

mod builder;
mod context;
mod genesis;
pub mod metrics;

/// The node for SpaceJam
pub struct Spacejam<S: Storage, V: Validator> {
    /// The context of the node
    pub context: Context<S, V>,

    /// The network of the node
    pub network: Network,
}

impl<S: Storage, V: Validator> Spacejam<S, V> {
    /// Create a new builder
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Start the node
    ///
    /// TODO: make metrics service out of this function?
    pub async fn start(mut self, metrics: SocketAddr) -> anyhow::Result<()> {
        tokio::select! {
            _ = metrics::serve(metrics, self.context.metrics.clone()) => {}
            _ = authoring(&self.context) => {}
            _ = self.network.spawn(&self.context) => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        Ok(())
    }
}

/// Author blocks (mocked)
async fn authoring<S: Storage, V: Validator>(context: &Context<S, V>) {
    let do_author = || async move {
        if let Some(block) = context.runtime.try_author().await? {
            tracing::info!(
                "subscribing block@{}: {}",
                block.header.slot,
                hex::encode(block.hash()?)
            );
            context
                .tx
                .send(Event::SubscribeBlock(codec::encode(&block)?))
                .await?;
        }

        Ok::<_, anyhow::Error>(())
    };

    loop {
        tokio::time::sleep(Duration::from_secs(score::SLOT_PERIOD as u64)).await;
        if let Err(e) = do_author().await {
            tracing::error!("failed to author block: {e}");
        }
    }
}
