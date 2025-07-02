//! Announcement handler
//!
//! Maintain the known leaves of the chain (descendants of the latest
//! finalized block with no known children).

use crate::{peer::Connection, stream::ext::Write, Network};
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use score::block::{Head, Header};

/// Announce the block to the peer.
#[tracing::instrument(skip_all, fields(peer = %conn.address.peer_id), name = "up0")]
pub async fn spawn<C: runtime::Config>(
    runtime: Network<C>,
    send: SendStream,
    recv: RecvStream,
    conn: Connection,
) -> Result<()> {
    let r = tokio::select! {
        r = self::send(runtime.clone(), send, conn.clone()) => r,
        r = self::recv(runtime.clone(), recv, conn.clone()) => r,
    };

    if let Err(e) = r {
        let _ = runtime
            .disconnect(conn.address.peer_id, e.to_string())
            .await?;
    }

    Ok(())
}

/// Announce the block to the peer.
#[tracing::instrument(skip_all)]
pub async fn send<C: runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    conn: Connection,
) -> anyhow::Result<()> {
    let mut rx = runtime.announce.subscribe();
    while let Ok(header) = rx.recv().await {
        let handshake = conn.handshake.read().await;
        if !handshake.accept(&header.hash()?) {
            continue;
        }

        // check if the block is acceptable for the remote peer.
        let local = runtime.handshake().await;
        let data = (header, local.head.clone());
        data.write(&mut send).await?;
        send.finish()?;
    }

    anyhow::bail!("announcement sender stream closed");
}

/// Receive the block announcement from a remote peer.
///
/// TODO:
///
/// - avoid validating the same header for several times.
#[tracing::instrument(skip_all)]
pub async fn recv<C: runtime::Config>(
    runtime: Network<C>,
    mut recv: RecvStream,
    conn: Connection,
) -> anyhow::Result<()> {
    loop {
        // 1. read the length of the announcement
        let mut buf = [0; 4];
        recv.read_exact(&mut buf).await?;
        let length = u32::from_le_bytes(buf);

        // 2. decode the announcement
        let mut buf = vec![0; length as usize];
        recv.read_exact(&mut buf).await?;
        let (header, head) = codec::decode::<(Header, Head)>(buf.as_ref())?;

        // 3. update the remote peer's handshake data.
        let lhead = header.head()?;
        let exists = {
            let mut handshake = conn.handshake.write().await;
            handshake.head = head;
            runtime
                .add_leaf_to(lhead.clone(), &header, &mut handshake)
                .await?
        };

        // 4. validate the header
        if exists {
            tracing::trace!(
                "block#{}@0x{} is already in the chain",
                header.slot,
                hex::encode(&lhead.hash.as_ref()[..3])
            );
            continue;
        }

        // 5. queue the block for requesting.
        {
            if runtime.queue.read().await.contains(&lhead.hash) {
                continue;
            } else {
                runtime.queue.write().await.insert(lhead.hash);
            }
        }

        // 6.trace the announcement data.
        {
            let handshake = conn.handshake.read().await.clone();
            tracing::trace!(
                "block#{}@0x{}, remote#{}@0x{}",
                header.slot,
                hex::encode(&lhead.hash.as_ref()[..3]),
                handshake.head.slot,
                hex::encode(&handshake.head.hash.as_ref()[..3]),
            );
        }

        // 7. request the block
        runtime.request(&header).await?;
    }
}
