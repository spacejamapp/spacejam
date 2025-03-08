//! Peer events handler

use crate::{
    peer::{Connection, PeerId},
    stream, Address, Network,
};
use quinn::VarInt;
use score::runtime::Validator;

/// Handle the connected event.
pub async fn connected<C: score::runtime::Config>(runtime: Network<C>, conn: Connection) {
    let pool = runtime.pool.clone();
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
        let is_validator = !grandpa
            .grid
            .neighbours(runtime.validator.ed25519_public_key())
            .is_empty();
        let grandpa = runtime.grandpa.read().await.clone();
        let neighbours = grandpa.grid.neighbours(address.peer_id.into());

        if is_validator && !neighbours.contains(&runtime.validator.ed25519_public_key()) {
            tracing::warn!("peer is not a neighbour, skipping up0 stream");
            return;
        }

        let address = address.clone();
        tokio::spawn(async move {
            if let Err(e) = stream::up0::send(runtime.clone(), address.peer_id).await {
                tracing::warn!("failed to send up0 stream: {e:?} for {address}");
            }
        });
    }

    // 4. insert the connection into the manager
    pool.write().await.insert(address.peer_id, conn.clone());
    tracing::debug!("connected to {}", address);
}

/// Handle the closed event.
pub async fn closed<C: score::runtime::Config>(
    runtime: Network<C>,
    peer: PeerId,
    reason: String,
) -> anyhow::Result<Option<Address>> {
    let pool = runtime.pool.clone();
    let Some(conn) = pool.write().await.remove(&peer) else {
        return Ok(None);
    };

    let address = Address::new(conn.remote_address(), peer);
    tracing::warn!("closing connection {address} with reason: {reason}");

    // close the connection in the pool and metrics
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
///
/// TODO: introduce configuration for the number of errors before closing the connection.
async fn serve<C: score::runtime::Config>(conn: Connection, runtime: Network<C>) {
    while let Ok((send, recv)) = conn.accept_bi().await {
        let runtime = runtime.clone();
        tokio::spawn(async move {
            if let Err(e) = stream::recv(conn.address.peer_id, send, recv, runtime).await {
                tracing::warn!("error with peer: {}: {e:?}", conn.address.peer_id);
            }
        });
    }
}
