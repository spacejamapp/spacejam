//! Block request stream.

use crate::{stream::ext::Write, Connection, Network};
use quinn::{RecvStream, SendStream, VarInt};
use runtime::storage::SyncStorage;
use score::OpaqueHash;
use serde::{Deserialize, Serialize};

impl<C: runtime::Config> Network<C> {
    /// Receive a block request.
    #[tracing::instrument(skip_all, name = "ce128::recv", parent = None)]
    pub async fn recv_ce128(
        &self,
        mut send: SendStream,
        mut recv: RecvStream,
    ) -> anyhow::Result<()> {
        let mut buf = [0; 4];
        recv.read_exact(&mut buf).await?;
        let length = u32::from_le_bytes(buf);
        if length != 37 {
            anyhow::bail!("invalid length of block request message, expected 37, got {length}");
        }

        let mut buf = [0; 37];
        recv.read_exact(&mut buf).await?;

        // parse the block request
        let request: Request = codec::decode(&buf)?;
        tracing::trace!(
            "received block request for block @{}, direction={}, maximum={}",
            hex::encode(&request.hash[..3]),
            request.direction,
            request.maximum,
        );
        let grandpa = self.grandpa().await;
        let lookup = grandpa.lookup(request.hash, request.direction, request.maximum);

        // fetch and write the blocks
        for (hash, _header) in lookup {
            let Ok(block) = self.storage.block(&hash) else {
                break;
            };
            block.write(&mut send).await?;
            tracing::trace!(
                "sent block#{}@{}",
                block.header.slot,
                hex::encode(&block.header.hash()?[..3])
            );
        }

        send.stopped().await?;
        send.finish()?;
        recv.stop(VarInt::from_u32(0))?;
        Ok(())
    }
}

/// Send a block request.
#[tracing::instrument(skip_all, fields(peer = ?conn.address.peer_id), name="ce128::send", parent = None)]
pub async fn send(conn: &Connection, request: Request) -> anyhow::Result<RecvStream> {
    let (mut send, recv) = conn.open_bi().await?;
    send.write(&[128]).await?;
    request.write(&mut send).await?;

    send.stopped().await?;
    send.finish()?;
    Ok(recv)
}

/// A block request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
pub struct Request {
    /// The hash of the block.
    pub hash: OpaqueHash,

    /// The direction of the block, 0 for ascending exclusive,
    /// 1 for descending inclusive.
    ///
    /// * Ascending exclusive: The sequence of blocks in the response should start
    ///   with a child of the given block, followed by a grandchild, and so on.
    /// * Descending inclusive: The sequence of blocks in the response should start
    ///   with the given block, followed by its parent, grandparent, and so on.
    pub direction: u8,

    /// The maximum number of blocks to request.
    pub maximum: u32,
}
