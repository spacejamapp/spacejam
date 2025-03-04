//! Connection request handlers

use crate::{stream, Network};
use std::collections::HashSet;

/// Handle block request.
pub async fn blocks<C: score::runtime::Config>(
    runtime: Network<C>,
    peer: [u8; 32],
    conn: quinn::Connection,
    request: stream::ce128::Request,
    finalize: bool,
) -> anyhow::Result<()> {
    let blocks = stream::ce128::send(conn, request).await?;
    for block in blocks {
        // announce block on receiving.
        let head = runtime.grandpa.read().await.head.clone();
        runtime
            .announce
            .send(codec::encode(&(block.header.clone(), head.clone()))?)?;

        // vote for leaf if it is
        if block.header.parent == head.hash {
            runtime
                .grandpa
                .write()
                .await
                .leaves
                .entry(head)
                .or_insert_with(HashSet::new)
                .insert(peer);
        }

        if !finalize {
            continue;
        }

        // Finalize the block if is syncing.
        if let Err(e) = runtime.finalize(&block).await {
            tracing::error!("failed to finalize block#{}: {}", block.header.slot, e);
            break;
        }

        // announce the latest finalized block
        let head = runtime.grandpa.read().await.head.clone();
        runtime
            .announce
            .send(codec::encode(&(block.header.clone(), head.clone()))?)?;
    }

    Ok(())
}
