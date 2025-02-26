//! Events for peers.

use crate::{
    peer::{Address, Manager},
    stream, Context,
};
use quinn::Connection;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Events for peers.
pub enum Event {
    /// A new peer has connected.
    Connected {
        /// The peer's public key.
        peer: [u8; 32],

        /// The connection.
        connection: Connection,

        /// Whether we should open the up0 stream.
        open_up0: bool,
    },

    /// A peer has disconnected.
    Closed {
        /// The peer id
        peer: [u8; 32],
    },
}

impl From<Event> for crate::Event {
    fn from(event: Event) -> Self {
        crate::Event::Peer(event)
    }
}

impl Event {
    /// Handle the event.
    pub async fn handle<C: Context + Send + Sync + 'static>(
        &self,
        context: Arc<C>,
    ) -> anyhow::Result<()> {
        let manager = context.manager();

        match self {
            Self::Connected {
                peer,
                connection,
                open_up0,
            } => {
                let address = Address::new(connection.remote_address(), peer);
                tracing::debug!("connected to {}", address);

                // 1. insert the connection into the manager
                manager.write().await.insert(*peer, connection.clone());

                // 2. establish the connection in the metrics
                context
                    .metrics()
                    .conn
                    .establish_connection(address.to_string());

                // 3. spawn the connection
                let ptx = manager.read().await.ptx.clone();
                tokio::spawn(Self::spawn_conn(
                    *peer,
                    connection.clone(),
                    ptx,
                    context.clone(),
                    manager.clone(),
                ));

                // 4. open the up0 stream if needed
                if *open_up0 {
                    let (send, recv) = connection.open_bi().await?;
                    tokio::spawn(async move {
                        if let Err(e) = stream::up0::send(send, recv, context, manager).await {
                            tracing::warn!("failed to send up0 stream: {e:?} for {address}");
                        }
                    });
                }
            }
            Self::Closed { peer } => {
                // 1. remove the connection from the manager
                let Some(conn) = manager.write().await.conns.remove(peer) else {
                    tracing::warn!("connection already closed");
                    return Ok(());
                };

                let address = Address::new(conn.remote_address(), peer);
                tracing::debug!("disconnected from {}", address);

                // 2. close the connection in the metrics
                context.metrics().conn.close_connection(address.to_string());
            }
        }

        Ok(())
    }

    /// Spawn a connection.
    async fn spawn_conn<C: Context>(
        peer: [u8; 32],
        conn: Connection,
        ptx: mpsc::UnboundedSender<Event>,
        context: Arc<C>,
        manager: Arc<RwLock<Manager>>,
    ) {
        while let Ok((send, recv)) = conn.accept_bi().await {
            if let Err(e) = stream::recv(send, recv, context.clone(), manager.clone()).await {
                tracing::warn!("failed to handle stream: {e:?}");
                continue;
            }
        }

        if let Err(e) = ptx.send(Event::Closed { peer }) {
            tracing::warn!("failed to send closed event: {e:?}");
        }
    }
}
