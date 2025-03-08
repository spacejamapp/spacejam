//! Block announcement stream.

use crate::{peer::PeerId, Event, Network};
pub use handshake::Handshake;
use quinn::{RecvStream, SendStream};
use score::{
    block::Header,
    runtime::{Head, Runtime},
    OpaqueHash, TimeSlot,
};
use std::sync::Arc;
use tokio::sync::RwLock;

mod announce;
mod handshake;

/// Send a block announcement.
pub async fn send<C: score::runtime::Config>(
    runtime: Network<C>,
    peer: PeerId,
) -> anyhow::Result<()> {
    let conn = runtime.get_conn(peer).await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    // 1. send the handshake
    let grandpa = runtime.runtime.grandpa.read().await;
    let handshake = grandpa.handshake();
    let mut buf = vec![0];
    buf.extend_from_slice(&handshake);
    send.write(&buf).await?;

    // 2. verify that we can receive handshake
    let handshake = Handshake::read(&mut recv).await?;
    conn.handshake.write().await.head = handshake.head;

    // 3. announcement loop
    let runtime = runtime.clone();
    tokio::spawn(async move {
        announce::unchecked(runtime.clone(), send, recv, conn).await;
    });

    Ok(())
}

/// Receive a block announcement
pub async fn recv<C: score::runtime::Config>(
    peer: PeerId,
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let conn = runtime.get_conn(peer).await?;

    // 1. read the grandpa data
    let handshake = Handshake::read(&mut recv).await?;
    conn.handshake.write().await.head = handshake.head;

    // 2. send the handshake data.
    let grandpa = runtime.runtime.grandpa.read().await;
    send.write(&grandpa.handshake()).await?;

    // 3. announcement loop.
    let runtime = runtime.clone();
    tokio::spawn(async move {
        announce::unchecked(runtime.clone(), send, recv, conn).await;
    });

    Ok(())
}
