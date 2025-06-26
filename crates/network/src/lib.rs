//! Network implementation of Spacejam.

use peer::PeerId;
use quinn::Endpoint;
use runtime::{storage::SyncStorage, Runtime, Storage, Validator};
use score::block::{Head, Header};
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::{broadcast, RwLock};
pub use {
    config::Config,
    peer::{Address, Connection},
    transport::Builder as TransportBuilder,
};

pub mod action;
mod config;
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

    /// (deprecated) The bootnodes of the network
    pub bootnode: Option<Address>,

    /// The announce channel of the network
    announce: broadcast::Sender<Header>,
}

impl<C: runtime::Config + Send + Sync + 'static> Clone for Network<C> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            runtime: self.runtime.clone(),
            pool: self.pool.clone(),
            bootnode: self.bootnode.clone(),
            announce: self.announce.clone(),
        }
    }
}

impl<C: runtime::Config + Send + Sync + 'static> Network<C> {
    /// Create a new network
    pub async fn new(config: Config, runtime: Arc<Runtime<C>>) -> anyhow::Result<Self> {
        let keypair = runtime.validator.ed25519().unwrap_or_default();
        let transport = transport::builder(keypair)
            .address(config.address)
            .genesis(config.genesis)
            .build()?;

        let this = Self {
            transport,
            runtime,
            pool: Arc::new(RwLock::new(Default::default())),
            bootnode: config.bootnode,
            announce: broadcast::channel(256).0,
        };

        Ok(this)
    }

    /// Lookup the best head from the network
    pub async fn lookup(&self, best: &Head) -> Vec<Connection> {
        let grandpa = self.runtime.grandpa.read().await;
        let pool = self.pool.read().await.clone();
        let mut feeds = Vec::new();
        for conn in pool.values() {
            let handshake = conn.handshake.read().await;

            // check if the connection is a feedi
            if handshake.head.hash == best.hash
                || grandpa
                    .ancestry
                    .is_descendant_of(&handshake.head.hash, &best.hash)
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

            self.connect(conn).await;
        }
    }

    /// Dial a new connection
    pub async fn dial(&self, addr: Address) -> anyhow::Result<()> {
        let conn = self
            .transport
            .connect(addr.address, addr.peer_id.to_string().as_str())?
            .await
            .map_err(|e| anyhow::anyhow!("failed to dial {addr}: {e}"))?;

        // we need to verify the peer id before sending the connected event
        let Ok(conn) = Connection::new(conn.clone(), true) else {
            conn.close(1u32.into(), "failed to verify alpn".as_bytes());
            anyhow::bail!("failed to verify alpn of {addr}");
        };

        self.connect(conn).await;
        Ok(())
    }

    /// Dial the validators
    pub async fn dial_validators(&self) {
        let me = self.me();
        let pool = self.pool.read().await.clone();
        let Ok(validators) = self.runtime.storage.current_validators() else {
            tracing::warn!("failed to get validators from storage");
            return;
        };

        for validator in validators {
            let key = validator.ed25519;
            let peer = PeerId::from(key);

            if key == me
                || (key[31] > 127 && me[31] > 127 && (key < me))
                || pool.contains_key(&peer)
            {
                continue;
            }

            let Some(ipv4) = validator.ipv4() else {
                tracing::warn!("validator {peer} is not reachable via IPv4");
                continue;
            };

            let address = Address::new(ipv4, peer);
            if let Err(e) = self.dial(address).await {
                tracing::warn!("failed to dial bootstrap peer: {e}");
            }
        }
    }

    /// Get a connection from the pool
    pub(crate) async fn get_conn(&self, peer: PeerId) -> anyhow::Result<Connection> {
        let Some(conn) = self.pool.read().await.get(&peer).cloned() else {
            self.disconnect(peer, "clean died connection".to_string())
                .await?;
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
