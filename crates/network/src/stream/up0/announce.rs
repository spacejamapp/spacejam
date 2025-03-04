//! Announcement handler

use crate::{stream::up0::Sync, Network};
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
    sync: Sync,
) {
    let sync = Arc::new(RwLock::new(sync));
    let r = tokio::select! {
        r = self::send(runtime.clone(), send, sync.clone()) => r,
        r = self::recv(runtime.clone(), recv, sync) => r,
    };

    if let Err(e) = r {
        runtime.transport.close(peer, e.to_string()).await;
    }
}

/// Announce the block to the peer.
pub async fn send<C: score::runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    mut sync: Arc<RwLock<Sync>>,
) -> anyhow::Result<()> {
    let mut rx = runtime.announce.subscribe();
    while let Ok(announce) = rx.recv().await {
        let (header, head): (Header, Head) = codec::decode(&announce)?;
        let grandpa = runtime.runtime.grandpa.read().await;
        let sync = sync.read().await;

        // 1. A descendant of the block is announced instead of the block itself.
        //
        // TODO: grandchild, etc. should also be handled.
        if sync.head.slot > head.slot {
            continue;
        }

        // 2. The block is not a descendant of the latest finalized block.
        //
        // TODO: grandchild, etc. should also be handled.
        if header.parent != sync.head.hash {
            continue;
        }

        // 3. The block, or a descendant of the block, has been announced by the other side of the stream.
        if sync.leaves.iter().any(|leaf| leaf.hash == head.hash) {
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
    mut sync: Arc<RwLock<Sync>>,
) -> anyhow::Result<()> {
    while let Ok(Some(chunk)) = recv.read_chunk(1, true).await {
        let grandpa = runtime.grandpa.read().await;
        let (header, head): (Header, Head) = codec::decode(chunk.bytes.as_ref())?;

        // 1. we are receiving a new leaf.
        if grandpa.head == head && header.parent == grandpa.head.hash {
            sync.write().await.leaves.push(Head {
                hash: header.hash()?,
                slot: header.slot,
            });
            continue;
        }

        // TODO: handle other cases.
    }
    Ok(())
}
