//! Block announcement stream.

use crate::{peer::Manager, Context};
use quinn::{RecvStream, SendStream};
use score::{block::Header, OpaqueHash, TimeSlot};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Send a block announcement.
///
/// TODO: considering timedout?
pub async fn send<C: Context>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Arc<C>,
    manager: Arc<RwLock<Manager>>,
) -> anyhow::Result<()> {
    // 1. send the handshake
    let handshake = self::handshake(context.clone()).await?;
    send.write_all(&handshake).await?;

    // 2. verify that we can receive handshake
    //
    // TODO: verify the received hanshake
    let mut buf = vec![];
    recv.read(&mut buf).await?;

    // 3. announcement loop
    self::announce(send, manager.clone()).await;
    Ok(())
}

/// Receive a block announcement
pub async fn recv<C: Context>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Arc<C>,
    manager: Arc<RwLock<Manager>>,
) -> anyhow::Result<()> {
    // TODO: check the the buf presents handshake data.
    let mut buf = vec![];
    recv.read(&mut buf).await?;

    // 2. send the handshake data.
    let handshake = self::handshake(context.clone()).await?;
    send.write_all(&handshake).await?;

    // 3. announcement loop.
    self::announce(send, manager.clone()).await;
    Ok(())
}

/// handle received blocks
///
/// TODO: we need to peer id here to do the audit as well
async fn import<C: Context>(mut recv: RecvStream, context: Arc<C>) {
    loop {
        let mut buf = vec![];
        if let Err(e) = recv.read(&mut buf).await {
            tracing::warn!("failed to read block with up0: {e:?}");
            continue;
        }

        let Ok((header, hash, slot)) = codec::decode::<(Header, OpaqueHash, TimeSlot)>(&buf) else {
            tracing::warn!("failed to decode block with up0");
            continue;
        };

        // TODO: verify the header and then send a
        // request for the block if needed.
    }
}

/// create a loop for announcing blocks
async fn announce(mut send: SendStream, manager: Arc<RwLock<Manager>>) {
    let mut brx = manager.read().await.btx.subscribe();
    tokio::spawn(async move {
        while let Ok(announce) = brx.recv().await {
            let mut data = vec![0];
            data.extend_from_slice(&announce);
            send.write_all(&data).await;
        }
    });
}

async fn handshake<C: Context>(context: Arc<C>) -> anyhow::Result<Vec<u8>> {
    let grandpa = context.grandpa();
    let grandpa = grandpa.read().await;
    let mut handshake = vec![];
    handshake.extend_from_slice(grandpa.head.hash()?.as_ref());
    handshake.extend_from_slice(&grandpa.head.slot.to_le_bytes());
    handshake.extend_from_slice(&grandpa.leaves.len().to_le_bytes());
    for leaf in grandpa.leaves.iter() {
        handshake.extend_from_slice(leaf.hash()?.as_ref());
        handshake.extend_from_slice(&leaf.slot.to_le_bytes());
    }

    Ok(handshake)
}
