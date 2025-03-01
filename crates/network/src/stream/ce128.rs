//! Block request stream.

use std::mem;

use crate::Network;
use quinn::{RecvStream, SendStream};
use score::{Block, OpaqueHash};
use serde::{Deserialize, Serialize};

/// Send a block request.
pub async fn send(
    mut send: SendStream,
    mut recv: RecvStream,
    request: Request,
) -> anyhow::Result<Vec<Block>> {
    let mut buf = vec![0];
    buf.extend_from_slice(request.hash.as_ref());
    buf.extend_from_slice(&request.direction.to_le_bytes());
    buf.extend_from_slice(&request.maximum.to_le_bytes());
    send.write_all(&buf).await?;
    send.finish();

    // 2. receive the response
    let blocks = codec::decode(&recv.read_to_end(usize::MAX).await?)?;
    Ok(blocks)
}

/// Receive a block request.
pub async fn recv<C: score::runtime::Config>(
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let buf = mem::size_of::<Request>();
    let mut buf = vec![0; buf];
    recv.read_exact(&mut buf).await?;

    // TODO: verify if mem::size_of works here
    let request: Request = codec::decode(&buf[..])?;
    // let blocks = runtime.runtime.storage.fetch_blocks(request)?;
    //
    // TODO: fetch the blocks from the storage
    let blocks: Vec<Block> = vec![];
    send.write_all(&codec::encode(&blocks)?).await?;
    send.finish();
    Ok(())
}

/// A block request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
pub struct Request {
    /// The hash of the block.
    pub hash: OpaqueHash,

    /// The direction of the block.
    pub direction: u8,

    /// The maximum number of blocks to request.
    pub maximum: u32,
}
