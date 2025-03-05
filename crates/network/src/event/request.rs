//! Connection request handlers

use crate::{stream, Network};

/// Handle block request, as well as the newly received blocks. note that
/// the block request event will only be triggered when:
///
/// we are syncing blocks from the remote peer.
pub async fn blocks<C: score::runtime::Config>(
    runtime: Network<C>,
    conn: quinn::Connection,
    request: stream::ce128::Request,
) -> anyhow::Result<()> {
    let chain = {
        let should_reverse = request.direction == 1;
        let mut chain = stream::ce128::send(conn, request).await?;
        if should_reverse {
            chain.reverse();
        }

        chain
    };

    for block in chain {
        // Try to finalize the block.
        if let Err(e) = runtime.finalize(&block).await {
            tracing::error!("failed to finalize block#{}: {}", block.header.slot, e);
            break;
        }

        // Announce the valid block.
        let head = runtime.grandpa.read().await.head.clone();
        runtime
            .announce
            .send((block.header.clone(), head.clone()))?;
    }

    Ok(())
}
