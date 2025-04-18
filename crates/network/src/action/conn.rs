//! Peer events handler

use crate::{
    peer::{Connection, PeerId},
    Address, Network,
};
use quinn::VarInt;
use runtime::Validator;

impl<C: runtime::Config> Network<C> {
    /// Handle the connected event.
    #[tracing::instrument(skip_all, name = "connect", fields(peer = conn.address.peer_id.to_string()))]
    pub async fn connect(&self, conn: Connection) {
        let address = conn.address.clone();

        // 1. establish the connection in the metrics
        self.metrics.conn.establish_connection(address.to_string());

        // 2. spawn the connection
        let runtime = self.clone();
        let cloned_conn = conn.clone();
        tokio::spawn(async move { runtime.serve(cloned_conn).await });

        // 3. open the up0 stream if needed
        if conn.outgoing {
            let grandpa = self.grandpa.read().await.clone();
            let neighbours = grandpa.grid.neighbours(self.validator.ed25519_public_key());

            if neighbours.contains(address.peer_id.as_ref()) || neighbours.is_empty() {
                let address = address.clone();
                let runtime = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = runtime.send_up0(address.peer_id).await {
                        tracing::warn!("failed to send up0 stream: {e:?} for {address}");
                    }
                });
            } else {
                tracing::trace!("peer is not a neighbour, skipping up0 stream");
            }
        }

        // 4. insert the connection into the manager
        self.pool
            .write()
            .await
            .insert(address.peer_id, conn.clone());
        tracing::trace!("connected");
    }

    /// Handle the closed event.
    #[tracing::instrument(skip_all, name = "close", fields(peer = peer.to_string()))]
    pub async fn disconnect(
        &self,
        peer: PeerId,
        reason: String,
    ) -> anyhow::Result<Option<Address>> {
        let pool = self.pool.clone();
        let Some(conn) = pool.write().await.remove(&peer) else {
            return Ok(None);
        };

        tracing::warn!("closing connection with reason: {reason}");

        // close the connection in the pool and metrics
        let address = Address::new(conn.remote_address(), peer);
        conn.close(VarInt::from(0_u8), reason.as_bytes());
        self.metrics.conn.close_connection(address.to_string());

        // if the connection is incoming, we don't need to dial again
        if !conn.outgoing {
            return Ok(None);
        }

        // check if the peer is a validator
        let grandpa = self.grandpa.read().await.clone();
        if grandpa.grid.validators().contains(peer.as_ref()) {
            return Ok(Some(address));
        }

        Ok(None)
    }

    /// Serve a connection.
    async fn serve(&self, conn: Connection) {
        // TODO: use limited threads to serve the connection

        let peer_id = conn.address.peer_id;
        while let Ok((send, recv)) = conn.accept_bi().await {
            self.handle(peer_id, send, recv).await;
        }
    }
}
