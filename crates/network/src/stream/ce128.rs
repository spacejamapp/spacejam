//! Block request stream.

use crate::{Connection, Network};
use quinn::{RecvStream, SendStream};
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
        let mut buf = [0; 37];
        recv.read_exact(&mut buf).await?;

        let request: Request = codec::decode(&buf)?;
        let grandpa = self.grandpa.read().await;
        let lookup = grandpa.lookup(request.hash, request.direction, request.maximum);

        // fetch and write the blocks
        for (hash, _header) in lookup {
            let Ok(block) = self.storage.get_block(&hash) else {
                break;
            };
            send.write(&codec::encode(&block)?).await?;
        }

        send.finish()?;
        Ok(())
    }
}

/// Send a block request.
#[tracing::instrument(skip_all, fields(peer = ?conn.address.peer_id), name="ce128::send", parent = None)]
pub async fn send(conn: Connection, request: Request) -> anyhow::Result<(SendStream, RecvStream)> {
    let (mut send, recv) = conn.open_bi().await?;

    let mut buf = vec![128];
    buf.extend_from_slice(request.hash.as_ref());
    buf.extend_from_slice(&request.direction.to_le_bytes());
    buf.extend_from_slice(&request.maximum.to_le_bytes());
    send.write_all(&buf).await?;

    // returns the recv stream
    Ok((send, recv))
}

/// A block request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
pub struct Request {
    /// The hash of the block.
    pub hash: OpaqueHash,

    /// The direction of the block.
    ///
    /// 0 for ascending exclusive, 1 for descending inclusive.
    pub direction: u8,

    /// The maximum number of blocks to request.
    pub maximum: u32,
}
