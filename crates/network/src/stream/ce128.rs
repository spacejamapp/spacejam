//! Block request stream.

use crate::{Connection, Network};
use quinn::{RecvStream, SendStream};
use score::{runtime::storage::BlockStorage, Block, OpaqueHash};
use serde::{Deserialize, Serialize};
use std::mem;

/// Send a block request.
pub async fn send(conn: Connection, request: Request) -> anyhow::Result<RecvStream> {
    let (mut send, recv) = conn.open_bi().await?;

    let mut buf = vec![0];
    buf.extend_from_slice(request.hash.as_ref());
    buf.extend_from_slice(&request.direction.to_le_bytes());
    buf.extend_from_slice(&request.maximum.to_le_bytes());
    send.write_all(&buf).await?;
    send.finish();

    // returns the recv stream
    Ok(recv)
}

/// Receive a block request.
pub async fn recv<C: score::runtime::Config>(
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let Some(request) = recv.read_chunk(1, true).await? else {
        return Err(anyhow::anyhow!("failed to receive block request"));
    };

    let request: Request = codec::decode(&request.bytes)?;
    let current = runtime.storage.get_slot(&request.hash)?;
    let slots = if request.direction == 0 {
        (current..current + request.maximum)
    } else {
        (current - request.maximum..current)
    }
    .collect::<Vec<_>>();

    // Fetch blocks in batches of 10.
    let batch_size = 10;
    for chunk in slots.chunks(batch_size) {
        let blocks = runtime.storage.fetch_blocks(chunk)?;
        for block in blocks {
            send.write(&codec::encode(&block)?).await?;
        }
    }

    send.finish();
    Ok(())
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
