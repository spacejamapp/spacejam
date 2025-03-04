//! Block announcement stream.

use crate::{peer::PeerId, Network};
use handshake::Sync;
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
    peer: [u8; 32],
) -> anyhow::Result<()> {
    let conn = runtime.get_conn(peer).await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    // 1. send the handshake
    let grandpa = runtime.runtime.grandpa.read().await;
    let handshake = grandpa.handshake();
    let mut buf = vec![0];
    buf.extend_from_slice(&handshake);
    send.write_all(&buf).await?;

    // 2. verify that we can receive handshake
    let sync = Sync::read(&mut recv).await?;
    if !sync.verify(&runtime) {
        return Err(anyhow::anyhow!("invalid handshake data"));
    }

    // 3. announcement loop
    let runtime = runtime.clone();
    tokio::spawn(async move {
        announce::unchecked(peer, runtime.clone(), send, recv, sync).await;
    });
    Ok(())
}

/// Receive a block announcement
pub async fn recv<C: score::runtime::Config>(
    peer: [u8; 32],
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    // 1. read the grandpa data
    let sync = Sync::read(&mut recv).await?;
    if !sync.verify(&runtime) {
        return Err(anyhow::anyhow!("invalid handshake data"));
    }

    // 2. send the handshake data.
    let grandpa = runtime.runtime.grandpa.read().await;
    let handshake = grandpa.handshake();
    send.write_all(&handshake).await?;

    // 3. announcement loop.
    let runtime = runtime.clone();
    tokio::spawn(async move {
        announce::unchecked(peer, runtime.clone(), send, recv, sync).await;
    });

    Ok(())
}
