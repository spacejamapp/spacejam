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
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;

/// Announce the block to the peer.
pub async fn unchecked<C: score::runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    mut recv: RecvStream,
    conn: Connection,
) {
    let r = tokio::select! {
        r = self::send(runtime.clone(), send, conn.clone()) => r,
        r = self::recv(runtime.clone(), recv, conn.clone()) => r,
    };

    if let Err(e) = r {
        runtime
            .transport
            .close(conn.address.peer_id, e.to_string())
            .await;
    }
}

/// Announce the block to the peer.
pub async fn send<C: score::runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    conn: Connection,
) -> anyhow::Result<()> {
    let mut rx = runtime.announce.subscribe();
    while let Ok((header, head)) = rx.recv().await {
        let grandpa = runtime.runtime.grandpa.read().await;
        let handshake = conn.handshake.read().await;
        let hash = header.hash()?;

        // Skip if the block, or a descendant of the block, has been
        // announced by the other side of the stream.
        let leaves = handshake.leaves.iter().filter(|l| l.slot > header.slot);
        for leaf in leaves {
            if !grandpa.is_descendant_of(leaf.hash, hash) {
                continue;
            }
        }

        send.write_all(&codec::encode(&(header, head))?).await?;
    }

    Ok(())
}

/// Receive the block announcement from a remote peer.
pub async fn recv<C: score::runtime::Config>(
    runtime: Network<C>,
    mut recv: RecvStream,
    conn: Connection,
) -> anyhow::Result<()> {
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        let grandpa = runtime.grandpa.read().await;
        let (header, head): (Header, Head) = codec::decode(chunk.bytes.as_ref())?;
        let leaf = Head {
            hash: header.hash()?,
            slot: head.slot,
        };

        // verify if the header is invalid with the local finalized head.
        if let Err(e) = grandpa.verify(&header).await {
            tracing::warn!("header invalid: {}", e);
            continue;
        }

        // Add this header to local leaves
        runtime.grandpa.write().await.add_leave(leaf.clone());
        runtime.grandpa.write().await.save_header(header);

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
            runtime.send(Event::SelectBestChain { slot: head.slot });
        }
    }
    Ok(())
}
