//! Network implementation of Spacejam.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
pub use {
    config::Config,
    context::Context,
    event::Event,
    peer::{Address, Manager},
    transport::{Builder as TransportBuilder, Transport},
};

mod config;
mod context;
pub mod event;
pub mod peer;
mod stream;
pub mod transport;

/// The network protocol name of Spacejam.
pub const PROTOCOL: &str = "jamnp-s";

/// The network of Spacejam.
pub struct Network<C: Context + Send + Sync + 'static> {
    /// The transport of the network
    pub transport: Transport,

    /// The context of the network
    pub context: Arc<C>,

    /// The manager of the network
    pub manager: Arc<RwLock<Manager>>,
}

impl<C: Context + Send + Sync + 'static> Clone for Network<C> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            context: self.context.clone(),
            manager: self.manager.clone(),
        }
    }
}

impl<C: Context + Send + Sync + 'static> Network<C> {
    /// Create a new network
    pub async fn new(config: Config, context: Arc<C>) -> anyhow::Result<Self> {
        let transport = Transport::builder(context.keypair().unwrap_or_default())
            .address(config.address)
            .genesis(config.genesis)
            .build(context.tx().clone())?;

        // Spawn a task to handle bootstrap dialing
        let bootstrap = config.bootstrap;
        if !bootstrap.is_empty() {
            for peer in bootstrap {
                tracing::debug!("dialing bootstrap peer: {peer}");
                if let Err(e) = transport.dial(peer).await {
                    tracing::warn!("failed to dial bootstrap peer: {e}");
                }
            }
        } else {
            tracing::debug!("no bootstrap peers, skipping");
        }

        transport.clone().spawn().await?;
        Ok(Self {
            transport,
            manager: Arc::new(RwLock::new(Manager::new(context.tx().clone()))),
            context,
        })
    }

    /// Spawn a task to handle events
    pub async fn spawn(&self, mut rx: mpsc::UnboundedReceiver<Event>) {
        while let Some(event) = rx.recv().await {
            if let Err(e) = event.handle(self.clone()).await {
                tracing::error!("failed to handle event: {e}");
            }
        }
    }
}
