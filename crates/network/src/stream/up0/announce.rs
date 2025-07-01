//! Announcement handler
//!
//! Maintain the known leaves of the chain (descendants of the latest
//! finalized block with no known children).

use crate::{peer::Connection, stream::ext::Write, Network};
use anyhow::Result;
use quinn::{RecvStream, SendStream};
use runtime::storage::SyncStorage;
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
        let grandpa = runtime.grandpa().await;
        let handshake = conn.handshake.read().await;
        tracing::debug!("sending announcement: #{}", header.slot);

        // check if the block is acceptable for the remote peer.
        match grandpa.accept_remote(&header, &handshake).await {
            Ok(head) => {
                let hash = head.hash;
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
        let data = (header, grandpa.handshake.head.clone());
        data.write(&mut send).await?;
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
        let grandpa = runtime.grandpa().await;
        {
            let mut handshake = conn.handshake.write().await;
            handshake.head = head;
            grandpa.add_leaf_to(header.head()?, &mut handshake)?;
        }

        // 4. validate the header
        let hash = header.hash()?;
        if grandpa.ancestry.header(&hash).is_ok() {
            continue;
        }

        if let Err(e) = runtime.validate(&header).await {
            tracing::warn!(
                "failed to validate header#{}@0x{}: {e}.",
                header.slot,
                hex::encode(&hash[..3]),
            );
            if let Err(e) = runtime.fallback().await {
                tracing::error!("failed to fallback: {e}");
            }
            continue;
        }

        // 5.trace the announcement data.
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

        // TODO: import the block to the chain directly.
        //
        // 1. check if the parent can be tracked locally
        // 2. request the block
        // 3. import the block

        // 7. Add this header to local leave
        runtime.grandpa.write().await.add_leaf(header.clone())?;
        runtime.select_best_chain(header.slot).await?;
    }
}
