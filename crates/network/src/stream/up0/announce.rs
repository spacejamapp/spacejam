//! Announcement handler

use crate::{stream::up0::Handshake, Network};
use quinn::{RecvStream, SendStream};
use score::{block::Header, runtime::Head};
use std::sync::Arc;
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
        r = self::recv(runtime.clone(), recv, handshake) => r,
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
    while let Ok(announce) = rx.recv().await {
        let (header, head): (Header, Head) = codec::decode(&announce)?;
        let grandpa = runtime.runtime.grandpa.read().await;
        let handshake = handshake.read().await;

        // 1. A descendant of the block is announced instead of the block itself.
        if handshake.head.slot > head.slot {
            continue;
        }

        // 2. The block is not a descendant of the latest finalized block.
        if header.parent != handshake.head.hash {
            continue;
        }

        // 3. The block, or a descendant of the block, has been announced by the other side of the stream.
        if handshake.leaves.iter().any(|leaf| leaf.hash == head.hash) {
            continue;
        }

        send.write_all(&announce).await?;
    }

    Ok(())
}

/// Announce the block to the peer.
///
/// see also section 19 for the details of grandpa.
pub async fn recv<C: score::runtime::Config>(
    runtime: Network<C>,
    mut recv: RecvStream,
    mut handshake: Arc<RwLock<Handshake>>,
) -> anyhow::Result<()> {
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        let grandpa = runtime.grandpa.read().await;
        let (header, head): (Header, Head) = codec::decode(chunk.bytes.as_ref())?;

        // 1. we are receiving a new leaf.
        if grandpa.head == head && header.parent == grandpa.head.hash {
            handshake.write().await.leaves.push(Head {
                hash: header.hash()?,
                slot: header.slot,
            });
            continue;
        }

        // TODO: handle other cases.
    }
    Ok(())
}
