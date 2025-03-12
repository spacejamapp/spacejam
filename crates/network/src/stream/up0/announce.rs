//! Announcement handler
//!
//! Maintain the known leaves of the chain (descendants of the latest
//! finalized block with no known children).

use crate::{
    peer::{Connection, PeerId},
    stream::up0::Handshake,
    Event, Network,
};
use quinn::{RecvStream, SendStream};
use score::{block::Header, runtime::Head};
use std::{
    collections::HashSet,
    sync::{atomic::Ordering, Arc},
};
use tokio::sync::RwLock;

/// Announce the block to the peer.
#[tracing::instrument(skip_all, fields(peer = %conn.address.peer_id), name = "up0")]
pub async fn unchecked<C: score::runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    mut recv: RecvStream,
    conn: Connection,
) {
    conn.ready.store(true, Ordering::Relaxed);
    let r = tokio::select! {
        r = self::send(runtime.clone(), send, conn.clone()) => r,
        r = self::recv(runtime.clone(), recv, conn.clone()) => r,
    };

    conn.ready.store(false, Ordering::Relaxed);
    if let Err(e) = r {
        tracing::error!("closing connection with reason: {e}");
        runtime
            .transport
            .close(conn.address.peer_id, e.to_string())
            .await;
    }
}

/// Announce the block to the peer.
#[tracing::instrument(skip_all)]
pub async fn send<C: score::runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    conn: Connection,
) -> anyhow::Result<()> {
    let peer = conn.address.peer_id;
    let mut rx = runtime.announce.subscribe();

    while let Ok(header) = rx.recv().await {
        // check if the block is a descendant of the local finalized head.
        let grandpa = runtime.grandpa.read().await.clone();
        let handshake = conn.handshake.read().await;
        let hash = header.hash()?;
        let shash = hex::encode(&hash.as_ref()[..3]);

        // Skip if the block is not a descendant of the remote peer's
        // finalized head.
        if !grandpa.is_descendant_of(hash, handshake.head.hash) {
            continue;
        }

        // Skip if the block, or a descendant of the block, has been
        // announced by the other side of the stream.
        let mut leaves = handshake
            .leaves
            .iter()
            .filter(|l| l.slot >= handshake.head.slot);
        if leaves.any(|leaf| grandpa.is_descendant_of(leaf.hash, hash) || leaf.hash == hash) {
            continue;
        }

        tracing::trace!(
            "block#{}: 0x{}, grandpa#{}: 0x{}, remote#{}: 0x{}",
            header.slot,
            shash,
            grandpa.handshake.head.slot,
            hex::encode(&grandpa.handshake.head.hash.as_ref()[..3]),
            handshake.head.slot,
            hex::encode(&handshake.head.hash.as_ref()[..3]),
        );
        send.write_all(&codec::encode(&(header, grandpa.handshake.head))?)
            .await?;
    }

    anyhow::bail!("announcement sender stream closed");
}

/// Receive the block announcement from a remote peer.
#[tracing::instrument(skip_all)]
pub async fn recv<C: score::runtime::Config>(
    runtime: Network<C>,
    mut recv: RecvStream,
    conn: Connection,
) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        buffer.extend_from_slice(&chunk.bytes);
        let Ok((header, head)) = codec::decode::<(Header, Head)>(buffer.as_ref()) else {
            continue;
        };

        buffer.clear();
        // update the remote peer's handshake data.
        //
        // We're doing this directly because the remote peer will only
        // send us the headers as they have already been added to their
        // local leaves.
        let grandpa = runtime.grandpa.read().await.clone();
        {
            let mut handshake = conn.handshake.write().await;
            handshake.head = head.clone();
            grandpa.add_leaf_to(&header, &mut handshake)?;
            drop(handshake);
        }

        // if we already have this header, skip re-announcing it.
        //
        // This is actually should be checked from the sender side, but we
        // do it here for the sake of simplicity.
        if grandpa.header(&header.hash()?).is_some() {
            continue;
        }

        // trace the announcement data.
        let handshake = conn.handshake.read().await.clone();
        tracing::trace!(
            "block#{}: 0x{}, grandpa#{}: 0x{}, remote#{}: 0x{}",
            header.slot,
            hex::encode(&header.hash()?.as_ref()[..3]),
            grandpa.handshake.head.slot,
            hex::encode(&grandpa.handshake.head.hash.as_ref()[..3]),
            handshake.head.slot,
            hex::encode(&handshake.head.hash.as_ref()[..3]),
        );

        // verify if the header is invalid with the local finalized head.
        if let Err(e) = grandpa.verify(&header).await {
            tracing::warn!("{e}");
            continue;
        }

        // Add this header to local leaves
        runtime.grandpa.write().await.add_leaf(header.clone())?;

        // broadcast the header to the network
        runtime.announce.send(header.clone())?;

        // Indicates that we need to select the best chain.
        //
        // Try to select the best chain if the remote peer's finalized
        // head is greater than the local finalized head.
        if header.slot > grandpa.handshake.head.slot {
            if let Err(e) = runtime.send(Event::SelectBestChain { slot: header.slot }) {
                tracing::error!("failed to send select best chain event: {e}");
            }
        }
    }

    anyhow::bail!("announcement receiver stream closed");
}
