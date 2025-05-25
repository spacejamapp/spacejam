//! Announcement handler
//!
//! Maintain the known leaves of the chain (descendants of the latest
//! finalized block with no known children).

use crate::{peer::Connection, Network};
use quinn::{RecvStream, SendStream};
use score::block::{Head, Header};
use std::sync::atomic::Ordering;

/// Announce the block to the peer.
#[tracing::instrument(skip_all, fields(peer = %conn.address.peer_id), name = "up0")]
pub async fn unchecked<C: runtime::Config>(
    runtime: Network<C>,
    send: SendStream,
    recv: RecvStream,
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
        let _ = runtime.close(conn.address.peer_id, e.to_string()).await;
    }
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
        let grandpa = runtime.grandpa.read().await.clone();
        let handshake = conn.handshake.read().await;

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
        let data = (header, grandpa.handshake.head);
        let encoded = codec::encode(&data)?;
        send.write(&encoded.len().to_le_bytes()).await?;
        send.write(&encoded).await?;
    }

    anyhow::bail!("announcement sender stream closed");
}

/// Receive the block announcement from a remote peer.
///
/// TODO:
///
/// - avoid validating the same header for serveral times.
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
        let grandpa = runtime.grandpa.read().await.clone();
        {
            let mut handshake = conn.handshake.write().await;
            handshake.head = head.clone();
            grandpa.add_leaf_to(header.clone().try_into()?, &mut handshake)?;
        }

        // 4. validate the header
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
        tracing::trace!(
            "validated header: {}@0x{}",
            header.slot,
            hex::encode(&hash[..3])
        );

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

        // 6. skip if the header exists
        {
            let grandpa = runtime.grandpa.read().await.clone();
            if grandpa.ancestry.header(&hash).is_some() {
                continue;
            }
        }

        // 7. Add this header to local leaves
        //
        // Note that we don't verify the header here since we may
        // not have the parent of it.
        runtime.grandpa.write().await.add_leaf(header.clone())?;

        // // TODO: we should only broadcast the header only if we
        // // have fetched it.
        // //
        // // broadcast the header to the network
        // // runtime.send(Event::AnnounceBlock(Box::new(header.clone())))?;
        // runtime.announce(Box::new(header.clone())).await?;
    }
}
