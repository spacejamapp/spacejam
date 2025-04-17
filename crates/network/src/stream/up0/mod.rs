//! Block announcement stream.

use crate::{peer::PeerId, Network};
use quinn::{RecvStream, SendStream};
use runtime::{Handshake, Head, Runtime};
use score::{block::Header, OpaqueHash, TimeSlot};
use std::sync::Arc;
use tokio::sync::RwLock;

mod announce;

/// Send a block announcement.
pub async fn send<C: runtime::Config>(runtime: Network<C>, peer: PeerId) -> anyhow::Result<()> {
    let conn = runtime.get_conn(peer).await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    // 1. send the handshake
    let grandpa = runtime.runtime.grandpa.read().await;
    let handshake = grandpa.handshake.clone();
    let mut buf = vec![0];
    buf.extend_from_slice(&codec::encode(&handshake)?);
    send.write(&buf).await?;

    // 2. get the handshake from remote
    let handshake = self::handshake(&mut recv).await?;
    conn.handshake.write().await.head = handshake.head;

    // 3. announcement loop
    let runtime = runtime.clone();
    tokio::spawn(async move {
        announce::unchecked(runtime.clone(), send, recv, conn).await;
    });

    Ok(())
}

/// Receive a block announcement
pub async fn recv<C: runtime::Config>(
    peer: PeerId,
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let conn = runtime.get_conn(peer).await?;

    // 1. read the grandpa data
    let handshake = self::handshake(&mut recv).await?;
    conn.handshake.write().await.head = handshake.head;

    // 2. send the handshake data.
    let grandpa = runtime.runtime.grandpa.read().await;
    send.write(&codec::encode(&grandpa.handshake)?).await?;

    // 3. announcement loop.
    let runtime = runtime.clone();
    tokio::spawn(async move {
        announce::unchecked(runtime.clone(), send, recv, conn).await;
    });

    Ok(())
}

/// Read the handshake from the stream.
async fn handshake(recv: &mut RecvStream) -> anyhow::Result<Handshake> {
    let mut buf = vec![];
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        buf.extend_from_slice(&chunk.bytes);
        if let Ok(handshake) = codec::decode(&buf) {
            return Ok(handshake);
        }
    }

    anyhow::bail!("failed to read handshake");
}
