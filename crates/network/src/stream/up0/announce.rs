//! Announcement handler

use crate::{stream::up0::Handshake, Network};
use quinn::{RecvStream, SendStream};
use score::{block::Header, runtime::Head};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;

/// Announce the block to the peer.
pub async fn unchecked<C: score::runtime::Config>(
    peer: [u8; 32],
    runtime: Network<C>,
    mut send: SendStream,
    mut recv: RecvStream,
    handshake: Handshake,
) {
    let handshake = Arc::new(RwLock::new(handshake));
    let r = tokio::select! {
        r = self::send(runtime.clone(), send, handshake.clone()) => r,
        r = self::recv(runtime.clone(), recv, handshake, peer) => r,
    };

    if let Err(e) = r {
        runtime.transport.close(peer, e.to_string()).await;
    }
}

/// Announce the block to the peer.
pub async fn send<C: score::runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    mut handshake: Arc<RwLock<Handshake>>,
) -> anyhow::Result<()> {
    let mut rx = runtime.announce.subscribe();
    while let Ok((header, head)) = rx.recv().await {
        let grandpa = runtime.runtime.grandpa.read().await;
        let handshake = handshake.read().await;
        let leaf = Head {
            slot: header.slot,
            hash: header.hash()?,
        };

        // 1. A descendant of the block is announced instead of the block itself.
        if grandpa.leaves.contains_key(&leaf) {
            continue;
        }

        // 2. The block is not a descendant of the latest finalized block.
        if header.parent != handshake.head.hash {
            continue;
        }

        // 3. The block, or a descendant of the block, has been announced by the other side of the stream.
        if handshake.leaves.contains(&leaf) {
            continue;
        }

        send.write_all(&codec::encode(&(header, leaf))?).await?;
    }

    Ok(())
}

/// Receive the block announcement from a remote peer.
pub async fn recv<C: score::runtime::Config>(
    runtime: Network<C>,
    mut recv: RecvStream,
    mut handshake: Arc<RwLock<Handshake>>,
    peer: [u8; 32],
) -> anyhow::Result<()> {
    // TODO: we should verify the header first to see if it
    // could be finalized, (fallback keys stuff).

    // TODO: if the votes of a specific leaf is enough, we should
    // request the full block and see if it could be finalized.

    // TODO: if the newly received header is confirmed as the best chain,
    // we should request the ancestor of the given block.
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        let grandpa = runtime.grandpa.read().await;
        let (header, head): (Header, Head) = codec::decode(chunk.bytes.as_ref())?;

        // This block has already been announced.
        if grandpa.leaves.contains_key(&head) {
            continue;
        }

        // The block is not a descendant of the latest finalized block.
        if grandpa.head.hash != header.parent {
            continue;
        }

        // The block has already been announced in the remote peer.
        let remote = handshake.read().await;
        if remote.leaves.contains(&head) {
            continue;
        }

        // Add this header to the remote info.
        handshake.write().await.leaves.push(Head {
            hash: header.hash()?,
            slot: header.slot,
        });

        // TODO: verify the header.

        // Add this header to local leaves
        let mut grandpa = runtime.grandpa.write().await;
        grandpa
            .leaves
            .entry(head)
            .or_insert_with(HashSet::new)
            .insert(peer);
    }
    Ok(())
}
