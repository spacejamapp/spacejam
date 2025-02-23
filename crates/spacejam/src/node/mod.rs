//! Node for SpaceJam

use network::Network;
use score::runtime::{Storage, Validator};
use std::net::SocketAddr;
use std::sync::Arc;
pub use {builder::Builder, context::Context, genesis::Genesis};

mod author;
mod builder;
mod context;
mod genesis;
pub mod metrics;

/// The node for SpaceJam
pub struct Spacejam<S: Storage + Send + Sync + 'static, V: Validator + Send + Sync + 'static> {
    /// The context of the node
    pub context: Arc<Context<S, V>>,

    /// The network of the node
    pub network: Network,
}

impl<S: Storage + Send + Sync + 'static, V: Validator + Send + Sync + 'static> Spacejam<S, V> {
    /// Create a new builder
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Start the node
    ///
    /// TODO: make metrics service out of this function?
    pub async fn start(self, metrics: SocketAddr) -> anyhow::Result<()> {
        let context = self.context.clone();

        tokio::select! {
            _ = metrics::serve(metrics, self.context.metrics.clone()) => {}
            _ = author::run(&self.context) => {}
            _ = self.network.spawn(context) => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        Ok(())
    }
}
