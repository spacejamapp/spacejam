//! Block announcement stream.

use crate::{event::action, peer::Manager, Context, Network};
use quinn::{RecvStream, SendStream};
use score::{block::Header, OpaqueHash, TimeSlot};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Send a block announcement.
///
/// TODO: considering timedout?
pub async fn send<C: Context + Send + Sync + 'static>(
    peer: [u8; 32],
    mut send: SendStream,
    mut recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    // 1. send the handshake
    let handshake = self::handshake(context.context.clone()).await?;
    let mut buf = vec![0];
    buf.extend_from_slice(&handshake);
    send.write_all(&buf).await?;

    // 2. verify that we can receive handshake
    let mut reader = GrandpaReader::new(&mut recv);
    reader.read().await?;

    // 3. announcement loop
    self::announce(send, context.manager.clone()).await;
    Ok(())
}

/// Receive a block announcement
pub async fn recv<C: Context + Send + Sync + 'static>(
    mut send: SendStream,
    mut recv: RecvStream,
    context: Network<C>,
) -> anyhow::Result<()> {
    // 1. read the grandpa data
    let mut reader = GrandpaReader::new(&mut recv);
    reader.read().await?;

    // 2. send the handshake data.
    let handshake = self::handshake(context.context.clone()).await?;
    send.write_all(&handshake).await?;

    // 3. announcement loop.
    self::announce(send, context.manager.clone()).await;
    Ok(())
}

/// handle received blocks
///
/// TODO: we need to peer id here to do the audit as well
async fn import<C: Context + Send + Sync + 'static>(
    peer: [u8; 32],
    mut recv: RecvStream,
    context: Network<C>,
) {
    let manager = context.manager.read().await;
    loop {
        let mut buf = vec![];
        if let Err(e) = recv.read(&mut buf).await {
            tracing::warn!("failed to read block with up0: {e:?}");
            continue;
        }

        let Ok((header, hash, slot)) = codec::decode::<(Header, OpaqueHash, TimeSlot)>(&buf) else {
            tracing::warn!("failed to decode block with up0");
            continue;
        };

        // TODO:
        // - consider adding to recent blocks?
        // - when to request the following states directly?
        // - or import the requested blocks and calculate the states locally?
        // - maybe this should be configurable? or we need to double check the state each time
        // we have a calculation on a new block.
        /* if context.context.grandpa().child(header, hash, slot) {
            // TODO: do sth here depend on the current node configuration.
            //
            // - import to the recent history
        } else {
            tracing::warn!("invalid block with up0");
        } */
    }
}

/// create a loop for announcing blocks
async fn announce(mut send: SendStream, manager: Arc<RwLock<Manager>>) {
    let mut brx = manager.read().await.btx.subscribe();
    tokio::spawn(async move {
        while let Ok(announce) = brx.recv().await {
            let mut data = vec![0];
            data.extend_from_slice(&announce);
            send.write_all(&data).await;
        }
    });
}

/// Fetch the handshake data from the context.
async fn handshake<C: Context>(context: Arc<C>) -> anyhow::Result<Vec<u8>> {
    let grandpa = context.grandpa();
    let mut handshake = vec![];
    handshake.extend_from_slice(grandpa.head.hash()?.as_ref());
    handshake.extend_from_slice(&grandpa.head.slot.to_le_bytes());
    handshake.extend_from_slice(&grandpa.leaves.len().to_le_bytes());

    for leaf in grandpa.leaves.iter() {
        handshake.extend_from_slice(leaf.hash.as_ref());
        handshake.extend_from_slice(&leaf.slot.to_le_bytes());
    }

    Ok(handshake)
}

/// Grandpa reader
struct GrandpaReader<'r> {
    finalized: OpaqueHash,
    leaves: Vec<OpaqueHash>,
    reader: &'r mut RecvStream,
}

impl<'r> GrandpaReader<'r> {
    pub fn new(reader: &'r mut RecvStream) -> Self {
        Self {
            finalized: Default::default(),
            leaves: Default::default(),
            reader,
        }
    }

    /// Read the grandpa data
    pub async fn read(&mut self) -> anyhow::Result<()> {
        // 1. read the finalized hash
        let mut hash = [0; 32];
        self.reader.read(&mut hash).await?;
        self.finalized = hash;

        // 2. read the leaves len
        let mut len = [0; 4];
        self.reader.read(&mut len).await?;

        // 3. read the leaves
        let leaves_len = u32::from_le_bytes(len) as usize * 32;
        for _ in 0..leaves_len {
            let mut hash = [0; 32];
            self.reader.read(&mut hash).await?;
            self.leaves.push(hash);
        }

        Ok(())
    }
}
