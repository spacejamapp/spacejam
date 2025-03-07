//! Peer events handler

use crate::{
    peer::{Connection, PeerId},
    stream, Address, Network,
};
use quinn::VarInt;
use score::runtime::Validator;

/// Handle the connected event.
pub async fn connected<C: score::runtime::Config>(
    runtime: Network<C>,
    conn: Connection,
    outgoing: bool,
) {
    let pool = runtime.pool.clone();
    let address = conn.address.clone();
    tracing::debug!("connected to {}", address);

    // 1. insert the connection into the manager
    pool.write().await.insert(address.peer_id, conn.clone());

    // 2. establish the connection in the metrics
    runtime
        .metrics
        .conn
        .establish_connection(address.to_string());

    // 3. spawn the connection
    tokio::spawn(self::serve(conn, runtime.clone()));

    // 4. open the up0 stream if needed
    if outgoing {
        let is_validator = runtime.is_validator;
        let grandpa = runtime.grandpa.read().await.clone();
        let neighbours = grandpa.grid.neighbours(address.peer_id.into());

        if is_validator && !neighbours.contains(&runtime.validator.ed25519_public_key()) {
            tracing::warn!("peer is not a neighbour, skipping up0 stream");
            return;
        }

        tokio::spawn(async move {
            if let Err(e) = stream::up0::send(runtime.clone(), address.peer_id).await {
                tracing::warn!("failed to send up0 stream: {e:?} for {address}");
            }
        });
    }
}

/// Handle the closed event.
pub async fn closed<C: score::runtime::Config>(
    runtime: Network<C>,
    peer: PeerId,
    reason: String,
) -> anyhow::Result<()> {
    let pool = runtime.pool.clone();
    let Some(conn) = pool.write().await.remove(&peer) else {
        tracing::warn!("connection already closed");
        return Ok(());
    };

    let address = Address::new(conn.remote_address(), peer);
    tracing::warn!("closing connection {address} with reason: {reason}");

    // close the connection in the pool and metrics
    conn.close(VarInt::from(0_u8), reason.as_bytes());
    runtime.metrics.conn.close_connection(address.to_string());

    Ok(())
}

/// Serve a connection.
async fn serve<C: score::runtime::Config>(conn: Connection, runtime: Network<C>) {
    while let Ok((send, recv)) = conn.accept_bi().await {
        if let Err(e) = stream::recv(conn.address.peer_id, send, recv, runtime.clone()).await {
            runtime
                .transport
                .close(conn.address.peer_id, e.to_string())
                .await;
            continue;
        }
    }
}
