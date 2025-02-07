//! Node for SpaceJam

use ::metrics::Metrics;
use network::Network;
use score::{state::Storage, validator::Validator};
use std::{net::SocketAddr, sync::Arc};
pub use {builder::Builder, context::Context};

mod builder;
mod context;
pub mod metrics;

/// The node for SpaceJam
pub struct Spacejam<S: Storage, V: Validator> {
    /// The context of the node
    pub context: Context<S, V>,

    /// The metrics of the node
    ///
    /// TODO: handle feature metrics
    pub metrics: Arc<Metrics>,

    /// The network of the node
    pub network: Network,

    /// If the node is authoring blocks
    ///
    /// TODO: remove this after implementing validator selection.
    pub authoring: bool,
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
            _ = metrics::serve(metrics, self.metrics.clone()) => {}
            _ = self.network.spawn(&self.context) => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        Ok(())
    }
}
