//! Announcement handler
//!
//! Maintain the known leaves of the chain (descendants of the latest
//! finalized block with no known children).

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
        let hash = header.hash()?;

        // Skip if the block, or a descendant of the block, has been
        // announced by the other side of the stream.
        let leaves = handshake.leaves.iter().filter(|l| l.slot > header.slot);
        for leaf in leaves {
            if !grandpa.is_descendant_of(&leaf.hash, hash) {
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
    mut handshake: Arc<RwLock<Handshake>>,
    peer: [u8; 32],
) -> anyhow::Result<()> {
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        let grandpa = runtime.grandpa.read().await;
        let (header, head): (Header, Head) = codec::decode(chunk.bytes.as_ref())?;

        // verify if the header is invalid with the local finalized head.
        if let Err(e) = grandpa.verify(&header).await {
            tracing::warn!("header invalid: {}", e);
            continue;
        }

        // Add this header to local leaves
        runtime.grandpa.write().await.leaves.insert(head);

        // TODO: Check if the head is on a finalized chain:
        //
        // 1. has the finliazed block as an ancestor. (checked above)
        // 2. contains no unfinalized blocks where we see an equivocation
        //    (two valid blocks at the same timeslot).
        // 3. is considered audited (as defined in the auditing section,
        //    where either there are no negative judgments and a tranche
        //    shows positive judgments from required validators, or there
        //    are positive judgments from >2/3 of validators)

        // TODO: if we have enough votes on the given leaf (best chain), we
        // should request the block and make the finalization.
    }
    Ok(())
}
