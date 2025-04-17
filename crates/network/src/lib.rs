//! Network implementation of Spacejam.

use metrics::Metrics;
use peer::PeerId;
use quinn::Endpoint;
use runtime::{Head, Runtime, Validator};
use score::block::Header;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::{broadcast, RwLock};
pub use {
    config::Config,
    peer::{Address, Connection},
    transport::Builder as TransportBuilder,
};

mod config;
pub mod event;
pub mod peer;
mod stream;
pub mod transport;

/// The network protocol name of Spacejam.
pub const PROTOCOL: &str = "jamnp-s";

/// The network of Spacejam.
pub struct Network<C: runtime::Config> {
    /// The transport of the network
    pub transport: Endpoint,

    /// The context of the network
    pub runtime: Arc<Runtime<C>>,

    /// The manager of the network
    pub pool: Arc<RwLock<HashMap<PeerId, Connection>>>,

    /// The metrics of the network
    pub metrics: Metrics,

    /// The announce channel of the network
    announce: broadcast::Sender<Header>,
}

impl<C: runtime::Config + Send + Sync + 'static> Clone for Network<C> {
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

impl<C: runtime::Config + Send + Sync + 'static> Network<C> {
    /// Create a new network
    pub async fn new(config: Config, runtime: Arc<Runtime<C>>) -> anyhow::Result<Self> {
        let keypair = runtime.validator.ed25519().unwrap_or_default();
        let peer_id = PeerId::from(keypair.verifying.to_bytes());
        let address = Address::new(config.address, peer_id);
        let transport = transport::builder(keypair)
            .address(config.address)
            .genesis(config.genesis)
            .build()?;

        let this = Self {
            transport,
            runtime,
            pool: Arc::new(RwLock::new(Default::default())),
            metrics: Metrics::new(address.to_string().as_str()),
            announce: broadcast::channel(256).0,
        };

        // bootstrap dialing
        let bootstrap = config.bootstrap;
        if !bootstrap.is_empty() {
            let this = this.clone();
            tokio::spawn(async move {
                for peer in bootstrap {
                    if let Err(e) = this.dial(peer).await {
                        tracing::warn!("failed to dial bootstrap peer: {e}");
                    }
                }
            });
        } else {
            tracing::debug!("no bootstrap peers, skip dialing ...");
        }

        Ok(this)
    }

    /// Lookup the best head from the network
    pub async fn lookup(&self, best: &Head) -> Vec<Connection> {
        let grandpa = self.runtime.grandpa.read().await.clone();
        let pool = self.pool.read().await.clone();
        let mut feeds = Vec::new();
        for conn in pool.values() {
            let handshake = conn.handshake.read().await;

            // check if the connection is a feedi
            if handshake.head.hash == best.hash
                || grandpa.is_descendant_of(handshake.head.hash, best.hash)
                || handshake.leaves.contains(best)
            {
                feeds.push(conn.clone());
            }
        }

        // we trust the feeds since
        //
        // - they are peers that we've connected to (validators)
        // - the best head is at least a descendant of their finalized heads
        //
        // so we can directly fetch the missing blocks from the feeds.
        feeds.sort_by_key(|conn| conn.latency);
        feeds
    }

    /// Spawn a task to handle events
    pub async fn spawn(&self) {
        let runtime = self.clone();
        let transport = self.transport.clone();

        loop {
            let Some(conn) = transport.accept().await else {
                tracing::error!("endpoint is closed");
                break;
            };

            let Ok(conn) = conn
                .await
                .map_err(|e| tracing::warn!("failed to accept connection: {e:?}"))
            else {
                continue;
            };

            let Ok(conn) = Connection::new(conn, false).map_err(|e| {
                tracing::warn!("failed to verify alpn: {e:?}");
            }) else {
                continue;
            };

            event::conn::connect(runtime.clone(), conn).await;
        }
    }

    /// Dial a new connection
    pub async fn dial(&self, addr: Address) -> anyhow::Result<()> {
        let conn = self
            .transport
            .connect(addr.addr, addr.peer_id.to_string().as_str())?
            .await
            .map_err(|e| anyhow::anyhow!("failed to dial {addr}: {e}"))?;

        // we need to verify the peer id before sending the connected event
        let Ok(conn) = Connection::new(conn.clone(), true) else {
            conn.close(1u32.into(), "failed to verify alpn".as_bytes());
            anyhow::bail!("failed to verify alpn of {addr}");
        };

        event::conn::connect(self.clone(), conn).await;
        Ok(())
    }

    /// Close a connection
    pub async fn close(&self, peer: PeerId, reason: String) -> anyhow::Result<()> {
        if let Some(_address) = event::conn::disconnect(self.clone(), peer, reason.clone()).await? {
            // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

            // TODO: re-connect missing peers with an interval
            //
            // self.dial(address).await?;
        }

        Ok(())
    }

    /// Get a connection from the pool
    pub(crate) async fn get_conn(&self, peer: PeerId) -> anyhow::Result<Connection> {
        let Some(conn) = self.pool.read().await.get(&peer).cloned() else {
            tracing::trace!("closing connection for peer: {peer}");
            self.close(peer, "No connection found".to_string()).await?;
            return Err(anyhow::anyhow!("no connection found for peer: {peer}"));
        };

        Ok(conn)
    }
}

impl<C: runtime::Config> Deref for Network<C> {
    type Target = Runtime<C>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
