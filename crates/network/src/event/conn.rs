//! Peer events handler

use crate::{
    peer::{Connection, PeerId},
    stream, Address, Network,
};
use quinn::VarInt;
use score::runtime::Validator;

/// Handle the connected event.
#[tracing::instrument(skip_all, name = "connect", fields(peer = conn.address.peer_id.to_string()))]
pub async fn connected<C: score::runtime::Config>(runtime: Network<C>, conn: Connection) {
    let address = conn.address.clone();

    // 1. establish the connection in the metrics
    runtime
        .metrics
        .conn
        .establish_connection(address.to_string());

    // 2. spawn the connection
    tokio::spawn(self::serve(conn.clone(), runtime.clone()));

    // 3. open the up0 stream if needed
    if conn.outgoing {
        let grandpa = runtime.grandpa.read().await.clone();
        let neighbours = grandpa
            .grid
            .neighbours(runtime.validator.ed25519_public_key());

        if neighbours.contains(address.peer_id.as_ref()) || neighbours.is_empty() {
            let address = address.clone();
            let runtime = runtime.clone();
            tokio::spawn(async move {
                if let Err(e) = stream::up0::send(runtime.clone(), address.peer_id).await {
                    tracing::warn!("failed to send up0 stream: {e:?} for {address}");
                }
            });
        } else {
            tracing::trace!("peer is not a neighbour, skipping up0 stream");
        }
    }

    // 4. insert the connection into the manager
    runtime
        .pool
        .write()
        .await
        .insert(address.peer_id, conn.clone());
    tracing::trace!("connected");
}

/// Handle the closed event.
#[tracing::instrument(skip_all, name = "close", fields(peer = peer.to_string()))]
pub async fn closed<C: score::runtime::Config>(
    runtime: Network<C>,
    peer: PeerId,
    reason: String,
) -> anyhow::Result<Option<Address>> {
    let pool = runtime.pool.clone();
    let Some(conn) = pool.write().await.remove(&peer) else {
        return Ok(None);
    };

    tracing::warn!("closing connection with reason: {reason}");

    // close the connection in the pool and metrics
    let address = Address::new(conn.remote_address(), peer);
    conn.close(VarInt::from(0_u8), reason.as_bytes());
    runtime.metrics.conn.close_connection(address.to_string());

    // if the connection is incoming, we don't need to dial again
    if !conn.outgoing {
        return Ok(None);
    }

    // check if the peer is a validator
    let grandpa = runtime.grandpa.read().await.clone();
    if grandpa.grid.validators().contains(peer.as_ref()) {
        return Ok(Some(address));
    }

    Ok(None)
}

/// Serve a connection.
async fn serve<C: score::runtime::Config>(conn: Connection, runtime: Network<C>) {
    // TODO: use limited threads to serve the connection

    while let Ok((send, recv)) = conn.accept_bi().await {
        let runtime = runtime.clone();
        tokio::spawn(async move { stream::recv(conn.address.peer_id, send, recv, runtime).await });
    }
}
