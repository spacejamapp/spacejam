//! Node for SpaceJam

use network::{Event, Network};
use score::runtime::{Storage, Validator};
use std::net::SocketAddr;
pub use {builder::Builder, context::Context, genesis::Genesis};

mod author;
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
            _ = author::run(&self.context) => {}
            _ = self.network.spawn(&self.context) => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        Ok(())
    }
}
