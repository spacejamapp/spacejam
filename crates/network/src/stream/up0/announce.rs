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
#[tracing::instrument(skip_all, fields(peer = %conn.address.peer_id), name = "announcement")]
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

    while let Ok((header, head)) = rx.recv().await {
        // check if the block is a descendant of the local finalized head.
        let grandpa = runtime.grandpa.read().await.clone();
        let handshake = conn.handshake.read().await;
        let hash = header.hash()?;

        // Skip if the block, or a descendant of the block, has been
        // announced by the other side of the stream.
        let leaves = handshake.leaves.iter().filter(|l| l.slot > header.slot);
        for leaf in leaves {
            if grandpa.is_descendant_of(leaf.hash, hash) {
                continue;
            }
        }

        tracing::trace!("block#{}(0x{})", header.slot, hex::encode(hash.as_ref()),);
        send.write_all(&codec::encode(&(header, head))?).await?;
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
        let leaf = Head {
            hash: header.hash()?,
            slot: header.slot,
        };

        tracing::trace!("block#{}(0x{})", leaf.slot, hex::encode(leaf.hash.as_ref()),);

        // verify if the header is invalid with the local finalized head.
        let grandpa = runtime.grandpa.read().await.clone();
        if let Err(e) = grandpa.verify(&header).await {
            tracing::warn!("{e}");
            continue;
        }

        // Add this header to local leaves
        {
            let mut grandpa = runtime.grandpa.write().await;
            grandpa.add_leave(leaf.clone());
            grandpa.save_header(header);
        }

        // update the remote peer's handshake data.
        {
            let mut handshake = conn.handshake.write().await;
            handshake.head = head.clone();
            handshake.leaves.insert(leaf.clone());
        }

        // Indicates that we need to select the best chain.
        //
        // Try to select the best chain if the remote peer's finalized
        // head is greater than the local finalized head.
        if head.slot > grandpa.head.slot {
            if let Err(e) = runtime.send(Event::SelectBestChain { slot: head.slot }) {
                tracing::error!("failed to send select best chain event: {e}");
            }
        }
    }

    anyhow::bail!("announcement receiver stream closed");
}
