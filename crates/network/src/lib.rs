//! Network implementation of Spacejam.

use metrics::Metrics;
use peer::PeerId;
use score::{
    block::Header,
    runtime::{Head, Runtime, Validator},
};
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::{broadcast, mpsc, RwLock};
pub use {
    config::Config,
    event::Event,
    peer::{Address, Connection},
    transport::{Builder as TransportBuilder, Transport},
};

mod config;
pub mod event;
pub mod peer;
mod stream;
pub mod transport;

/// The network protocol name of Spacejam.
pub const PROTOCOL: &str = "jamnp-s";

/// The network of Spacejam.
pub struct Network<C: score::runtime::Config> {
    /// The transport of the network
    pub transport: Transport,

    /// The context of the network
    pub runtime: Arc<Runtime<C>>,

    /// The manager of the network
    pub pool: Arc<RwLock<HashMap<PeerId, Connection>>>,

    /// The metrics of the network
    pub metrics: Metrics,

    /// The announce channel of the network
    announce: broadcast::Sender<(Header, Head)>,
}

impl<C: score::runtime::Config + Send + Sync + 'static> Clone for Network<C> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            runtime: self.runtime.clone(),
            pool: self.pool.clone(),
            metrics: self.metrics.clone(),
            announce: self.announce.clone(),
        }
    }
}

impl<C: score::runtime::Config + Send + Sync + 'static> Network<C> {
    /// Create a new network
    pub async fn new(
        config: Config,
        runtime: Arc<Runtime<C>>,
        tx: mpsc::UnboundedSender<Event>,
    ) -> anyhow::Result<Self> {
        let keypair = runtime.validator.ed25519().unwrap_or_default();
        let peer_id = PeerId::from(keypair.verifying.to_bytes());
        let address = Address::new(config.address, peer_id);
        let transport = Transport::builder(keypair)
            .address(config.address)
            .genesis(config.genesis)
            .build(tx.clone())?;

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
            runtime,
            pool: Arc::new(RwLock::new(Default::default())),
            metrics: Metrics::new(address.to_string().as_str()),
            announce: broadcast::channel(256).0,
        })
    }

    /// Send an event to the network
    pub fn send(&self, event: Event) -> anyhow::Result<()> {
        self.transport.tx.send(event)?;
        Ok(())
    }

    /// Spawn a task to handle events
    pub async fn spawn(&self, mut rx: mpsc::UnboundedReceiver<Event>) {
        while let Some(event) = rx.recv().await {
            if let Err(e) = event.clone().handle(self.clone()).await {
                tracing::error!("failed to handle event {event}: {e}");
            }
        }
    }

    /// Get a connection from the pool
    pub(crate) async fn get_conn(&self, peer: PeerId) -> anyhow::Result<Connection> {
        let Some(conn) = self.pool.read().await.get(&peer).cloned() else {
            self.transport.tx.send(Event::Closed {
                peer,
                reason: "No connection found".to_string(),
            })?;
            return Err(anyhow::anyhow!("no connection found for peer: {peer}"));
        };

        Ok(conn)
    }
}

impl<C: score::runtime::Config> Deref for Network<C> {
    type Target = Runtime<C>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
