//! Network implementation of Spacejam.

use peer::PeerId;
use quinn::{Endpoint, VarInt};
use runtime::{Runtime, Storage, Validator};
use score::block::{Head, Header};
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::{broadcast, RwLock};
pub use {
    config::Config,
    peer::{Address, Connection},
    transport::Builder as TransportBuilder,
};

// pub mod action;
mod config;
pub mod peer;
mod stream;
mod sync;
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
    ///
    /// TODO: possibly cache the history of unfinalized remote blocks
    /// of each peer, wait for the refactor of grandpa.
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

    /// Handle the connected event.
    #[tracing::instrument(skip_all, name = "connect", fields(peer = conn.address.peer_id.to_string()))]
    pub async fn connect(&self, conn: Connection) {
        let address = conn.address.clone();

        // 1. establish the connection in the metrics
        // self.metrics.conn.establish_connection(address.to_string());

        // 2. spawn the connection
        let runtime = self.clone();
        let cloned_conn = conn.clone();
        tokio::spawn(async move { runtime.serve(cloned_conn).await });

        // 3. insert the connection into the manager
        self.pool
            .write()
            .await
            .insert(address.peer_id, conn.clone());

        // 4. open the up0 stream if needed
        if conn.outgoing {
            let grandpa = self.grandpa().await;
            let neighbours = grandpa.grid.neighbours(self.validator.ed25519_public_key());

            if neighbours.contains(address.peer_id.as_ref()) || neighbours.is_empty() {
                let address = address.clone();
                let runtime = self.clone();
                if let Err(e) = runtime.send_up0(address.peer_id).await {
                    tracing::warn!("failed to send up0 stream: {e:?} for {address}");
                }
            }
        }

        tracing::debug!("connection established");
    }

    /// Handle the closed event.
    #[tracing::instrument(skip_all, name = "disconnect", fields(peer = peer.to_string()))]
    pub async fn disconnect(
        &self,
        peer: PeerId,
        reason: String,
    ) -> anyhow::Result<Option<Address>> {
        tracing::debug!("{reason}");
        let pool = self.pool.clone();
        let Some(conn) = pool.write().await.remove(&peer) else {
            return Ok(None);
        };

        // close the connection in the pool and metrics
        let address = Address::new(conn.remote_address(), peer);
        conn.close(VarInt::from(0_u8), reason.as_bytes());
        // self.metrics.conn.close_connection(address.to_string());

        // if the connection is incoming, we don't need to dial again
        if !conn.outgoing {
            return Ok(None);
        }

        // check if the peer is a validator
        let grandpa = self.grandpa().await;
        if grandpa.grid.validators().contains(peer.as_ref()) {
            return Ok(Some(address));
        }

        Ok(None)
    }

    /// Dial a new connection
    pub async fn dial(&self, addr: &Address) -> anyhow::Result<()> {
        let conn = self
            .transport
            .connect(addr.address, addr.peer_id.to_string().as_str())?
            .await?;

        // we need to verify the peer id before sending the connected event
        let Ok(conn) = Connection::new(conn.clone(), true) else {
            conn.close(1u32.into(), "failed to verify alpn".as_bytes());
            anyhow::bail!("failed to verify alpn of {addr}");
        };

        self.connect(conn).await;
        Ok(())
    }

    /// Dial the validators
    ///
    /// TODO: verify the connections before dialing, ping?
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
            tracing::info!("dialing peer {address}");
            if let Err(e) = self.dial(&address).await {
                tracing::warn!("failed to dial {address}: {e}");
            }
        }
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

    /// Serve a connection.
    async fn serve(&self, conn: Connection) {
        let peer_id = conn.address.peer_id;
        loop {
            match conn.accept_bi().await {
                Ok((send, recv)) => {
                    self.handle(peer_id, send, recv).await;
                }
                Err(e) => {
                    if let Err(e) = self.disconnect(peer_id, e.to_string()).await {
                        tracing::error!("failed to disconnect: {e:?}");
                    }
                    break;
                }
            }
        }
    }
}

impl<C: runtime::Config> Deref for Network<C> {
    type Target = Runtime<C>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
