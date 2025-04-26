//! Announcement handler
//!
//! Maintain the known leaves of the chain (descendants of the latest
//! finalized block with no known children).

use crate::{
    peer::{Connection, PeerId},
    stream::up0::Handshake,
    Network,
};
use quinn::{RecvStream, SendStream};
use runtime::Head;
use score::block::Header;
use std::{
    collections::HashSet,
    sync::{atomic::Ordering, Arc},
};
use tokio::sync::RwLock;

/// Announce the block to the peer.
#[tracing::instrument(skip_all, fields(peer = %conn.address.peer_id), name = "up0")]
pub async fn unchecked<C: runtime::Config>(
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
        runtime.close(conn.address.peer_id, e.to_string()).await;
    }
}

/// Announce the block to the peer.
#[tracing::instrument(skip_all)]
pub async fn send<C: runtime::Config>(
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
                let handshake = conn.handshake.read().await;
                tracing::trace!(
                    "block#{}@0x{}, grandpa#{}@0x{}, remote#{}@0x{}",
                    header.slot,
                    shash,
                    grandpa.handshake.head.slot,
                    hex::encode(&grandpa.handshake.head.hash.as_ref()[..3]),
                    handshake.head.slot,
                    hex::encode(&handshake.head.hash.as_ref()[..3]),
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
pub async fn recv<C: runtime::Config>(
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
        let grandpa = runtime.grandpa.read().await.clone();
        {
            let mut handshake = conn.handshake.write().await;
            handshake.head = head.clone();
            grandpa.add_leaf_to(header.clone().try_into()?, &mut handshake)?;
        }

        // validate the header
        let hash = header.hash()?;
        if grandpa.ancestry.header(&hash).is_some() {
            continue;
        }

        if let Err(e) = runtime.validate(&header).await {
            tracing::warn!(
                "failed to validate header#{}@0x{}: {e}. \n\nTODO: if this is caused by the epoch, we should request the ancestors of the block then handle it",
                header.slot,
                hex::encode(&hash[..3]),
            );
            continue;
        }

        // trace the announcement data.
        {
            let handshake = conn.handshake.read().await.clone();
            tracing::trace!(
                "block#{}@0x{}, grandpa#{}@0x{}, remote#{}@0x{}",
                header.slot,
                hex::encode(&hash.as_ref()[..3]),
                grandpa.handshake.head.slot,
                hex::encode(&grandpa.handshake.head.hash.as_ref()[..3]),
                handshake.head.slot,
                hex::encode(&handshake.head.hash.as_ref()[..3]),
            );
        }

        // skip if the header exists
        {
            let grandpa = runtime.grandpa.read().await.clone();
            if grandpa.ancestry.header(&hash).is_some() {
                continue;
            }
        }

        // Add this header to local leaves
        //
        // Note that we don't verify the header here since we may
        // not have the parent of it.
        runtime.grandpa.write().await.add_leaf(header.clone())?;

        // broadcast the header to the network
        // runtime.send(Event::AnnounceBlock(Box::new(header.clone())))?;
        runtime.announce(Box::new(header.clone())).await?;
    }

    anyhow::bail!("announcement receiver stream closed");
}
