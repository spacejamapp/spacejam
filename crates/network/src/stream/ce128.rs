//! Block request stream.

use crate::{Connection, Network};
use quinn::{RecvStream, SendStream};
use score::{runtime::storage::BlockStorage, Block, OpaqueHash};
use serde::{Deserialize, Serialize};
use std::mem;

/// Send a block request.
#[tracing::instrument(skip_all, level = "debug", fields(peer = ?conn.address.peer_id), name="ce128::send")]
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

/// Receive a block request.
#[tracing::instrument(skip_all, level = "debug", name = "ce128::recv")]
pub async fn recv<C: score::runtime::Config>(
    mut send: SendStream,
    mut recv: RecvStream,
    runtime: Network<C>,
) -> anyhow::Result<()> {
    let mut buf = [0; 37];
    recv.read_exact(&mut buf).await?;

    // TODO: maybe support child relationship in ancestry
    let request: Request = codec::decode(&buf)?;
    let mut ancestors = {
        let grandpa = runtime.grandpa.read().await.clone();
        grandpa
            .ancestors(&request.hash, grandpa.head.hash)
            .iter()
            .filter_map(|(h, _)| {
                if *h == request.hash || *h == grandpa.head.hash {
                    None
                } else {
                    Some(*h)
                }
            })
            .collect::<Vec<_>>()
    };
    ancestors.shrink_to((request.maximum as usize).min(ancestors.len())); 
    tracing::trace!("request for {} blocks.", ancestors.len());

    // Fetch blocks in batches of 10.
    let batch_size = 10;
    let chain = runtime.chain().await;
    for batch in ancestors.chunks(batch_size) {
        let blocks = chain.fetch_blocks(batch)?;
        tracing::trace!("fetched {} blocks.", blocks.len());
        for block in blocks {
            send.write(&codec::encode(&block)?).await?;
        }
    }

    tracing::trace!("finishing stream.");
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
