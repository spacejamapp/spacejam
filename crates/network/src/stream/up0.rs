//! Block announcement stream.

use crate::{
    event::{action, conn},
    peer::PeerId,
    Network,
};
use quinn::{RecvStream, SendStream};
use score::{
    block::Header,
    runtime::{Head, Runtime},
    OpaqueHash, TimeSlot,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Send a block announcement.
///
/// TODO: considering timedout?
pub async fn send<C: score::runtime::Config>(
    runtime: Network<C>,
    peer: [u8; 32],
) -> anyhow::Result<()> {
    let conn = runtime.get_conn(peer).await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    // 1. send the handshake
    let grandpa = runtime.runtime.grandpa.read().await;
    let handshake = grandpa.handshake();
    let mut buf = vec![0];
    buf.extend_from_slice(&handshake);
    send.write_all(&buf).await?;

    // 2. verify that we can receive handshake
    let sync = Sync::read(&mut recv).await?;

    // 3. announcement loop
    let runtime = runtime.clone();
    tokio::spawn(async move {
        self::announce_unchecked(peer, runtime.clone(), send, recv, sync).await;
    });
    Ok(())
}

/// Receive a block announcement
pub async fn recv<C: score::runtime::Config>(
    peer: [u8; 32],
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    // 1. read the grandpa data
    let sync = Sync::read(&mut recv).await?;

    // 2. send the handshake data.
    let grandpa = runtime.runtime.grandpa.read().await;
    let handshake = grandpa.handshake();
    send.write_all(&handshake).await?;

    // 3. announcement loop.
    let runtime = runtime.clone();
    tokio::spawn(async move {
        self::announce_unchecked(peer, runtime.clone(), send, recv, sync).await;
    });

    Ok(())
}

/// Announce the block to the peer.
async fn send_announce<C: score::runtime::Config>(
    runtime: Network<C>,
    mut send: SendStream,
    mut sync: Arc<RwLock<Sync>>,
) -> anyhow::Result<()> {
    let mut rx = runtime.announce.subscribe();
    while let Ok(announce) = rx.recv().await {
        let (header, head): (Header, Head) = codec::decode(&announce)?;
        let grandpa = runtime.runtime.grandpa.read().await;

        sync.write().await.head = head;
    }

    Ok(())
}

/// Announce the block to the peer.
async fn recv_announce<C: score::runtime::Config>(
    runtime: Network<C>,
    mut recv: RecvStream,
    mut sync: Arc<RwLock<Sync>>,
) -> anyhow::Result<()> {
    Ok(())
}

async fn announce_unchecked<C: score::runtime::Config>(
    peer: [u8; 32],
    runtime: Network<C>,
    mut send: SendStream,
    mut recv: RecvStream,
    sync: Sync,
) {
    let sync = Arc::new(RwLock::new(sync));
    let r = tokio::select! {
        r = send_announce(runtime.clone(), send, sync.clone()) => r,
        r = recv_announce(runtime.clone(), recv, sync) => r,
    };

    if let Err(e) = r {
        runtime.transport.close(peer, e.to_string()).await;
    }
}

/// Sync information.
struct Sync {
    /// The finalized head.
    head: Head,

    /// The leaves.
    leaves: Vec<Head>,
}

impl Sync {
    /// Create a new sync information from the receiver stream.
    pub async fn read(recv: &mut quinn::RecvStream) -> anyhow::Result<Self> {
        // 1. read the finalized hash
        let mut hash = [0; 32];
        recv.read(&mut hash).await?;
        let mut slot = [0; 4];
        recv.read(&mut slot).await?;
        let head = Head {
            hash: OpaqueHash::from(hash),
            slot: TimeSlot::from_le_bytes(slot),
        };

        // 2. read the leaves len
        let mut len = [0; 4];
        recv.read(&mut len).await?;

        // 3. read the leaves
        let mut leaves = Vec::new();
        let leaves_len = u32::from_le_bytes(len) as usize * 32;
        for _ in 0..leaves_len {
            let mut hash = [0; 32];
            recv.read(&mut hash).await?;
            let mut slot = [0; 4];
            recv.read(&mut slot).await?;
            leaves.push(Head {
                hash: OpaqueHash::from(hash),
                slot: TimeSlot::from_le_bytes(slot),
            });
        }

        Ok(Self { head, leaves })
    }
}
