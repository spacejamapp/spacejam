//! Events for peers.

use crate::{peer::Address, stream, Network};
use quinn::VarInt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Events for peers.
pub enum Event {
    /// A new peer has connected.
    Connected {
        /// The peer's public key.
        peer: [u8; 32],

        /// The connection.
        connection: quinn::Connection,

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
    pub async fn handle<C: score::runtime::Config>(
        &self,
        runtime: Network<C>,
    ) -> anyhow::Result<()> {
        let pool = runtime.pool.clone();

        match self {
            Self::Connected {
                peer,
                connection,
                open_up0,
            } => {
                let address = Address::new(connection.remote_address(), peer);
                tracing::debug!("connected to {}", address);

                // 1. insert the connection into the manager
                pool.write()
                    .await
                    .insert(*peer, Arc::new(RwLock::new(connection.clone().into())));

                // 2. establish the connection in the metrics
                runtime
                    .metrics
                    .conn
                    .establish_connection(address.to_string());

                // 3. spawn the connection
                tokio::spawn(Self::serve(*peer, connection.clone(), runtime.clone()));

                // 4. open the up0 stream if needed
                if *open_up0 {
                    let peer = *peer;
                    tokio::spawn(async move {
                        if let Err(e) = stream::up0::send(runtime.clone(), peer).await {
                            tracing::warn!("failed to send up0 stream: {e:?} for {address}");
                        }
                    });
                }
            }
            Self::Closed { peer } => {
                // 1. remove the connection from the manager
                let Some(conn) = pool.write().await.remove(peer) else {
                    tracing::warn!("connection already closed");
                    return Ok(());
                };

                let address = Address::new(conn.read().await.conn.remote_address(), peer);
                tracing::debug!("disconnected from {}", address);

                let conn = conn.write().await;
                // TODO: close connection with reason.
                conn.close(VarInt::from(0_u8), "disconnected".as_bytes());

                // 2. close the connection in the metrics
                runtime.metrics.conn.close_connection(address.to_string());
            }
        }

        Ok(())
    }

    /// Serve a connection.
    async fn serve<C: score::runtime::Config>(
        peer: [u8; 32],
        conn: quinn::Connection,
        runtime: Network<C>,
    ) {
        let tx = runtime.transport.tx.clone();
        while let Ok((send, recv)) = conn.accept_bi().await {
            if let Err(e) = stream::recv(peer, send, recv, runtime.clone()).await {
                tracing::warn!("failed to handle stream: {e:?}");
                continue;
            }
        }

        if let Err(e) = tx.send(Event::Closed { peer }.into()) {
            tracing::warn!("failed to send closed event: {e:?}");
        }
    }
}
