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
        let grandpa = runtime.grandpa.read().await.clone();
        let handshake = conn.handshake.read().await;

        // check if the block is acceptable for the remote peer.
        match grandpa.accept_remote(&header, &handshake).await {
            Ok(head) => {
                let hash = header.hash()?;
                let shash = hex::encode(&hash.as_ref()[..3]);
                tracing::trace!(
                    "block#{}: 0x{}, grandpa#{}: 0x{}, remote#{}: 0x{}",
                    header.slot,
                    shash,
                    grandpa.handshake.head.slot,
                    hex::encode(&grandpa.handshake.head.hash.as_ref()[..3]),
                    head.slot,
                    hex::encode(&head.hash.as_ref()[..3]),
                );
            }
            Err(e) => {
                tracing::trace!("{e}");
                continue;
            }
        }

        // send the announcement to the remote peer.
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
        }

        if let Err(e) = runtime.importer().validate(&header).await {
            tracing::trace!("failed to validate header: {e}");
            continue;
        }

        // TODO: if the header is at the same slot with the finalized head,
        // validate the seal of it.
        //
        // validation logic below vv

        // trace the announcement data.
        {
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
        }

        // Add this header to local leaves
        //
        // Note that we don't verify the header here since we may
        // not have the parent of it.
        runtime.grandpa.write().await.add_leaf(header.clone())?;

        // broadcast the header to the network
        runtime.send(Event::AnnounceBlock(Box::new(header.clone())))?;

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
