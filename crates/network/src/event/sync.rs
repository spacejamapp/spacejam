//! Handler of sync events

use crate::{stream::ce128, Network};
use score::{Block, TimeSlot};

/// Select the best chain.
///
/// This happens on:
/// - receiving new block announcements
/// - before authoring blocks
#[tracing::instrument(
    skip_all,
    level = "debug",
    name = "finalizing",
    fields(slot = ?slot)
)]
pub async fn select_best_chain<C: score::runtime::Config>(
    runtime: Network<C>,
    slot: TimeSlot,
) -> anyhow::Result<()> {
    let grandpa = runtime.grandpa.read().await.clone();
    if slot <= grandpa.head.slot {
        return Ok(());
    }

    // select the best head from the grandpa.
    let Some(best) = grandpa.select_best_head() else {
        return Ok(());
    };

    let mut feeds = Vec::new();
    let pool = runtime.pool.read().await.clone();
    for conn in pool.values() {
        let head = conn.handshake.read().await.head.clone();
        if head.hash == best.hash
            || head.slot > best.slot && grandpa.is_descendant_of(head.hash, best.hash)
        {
            feeds.push(conn.clone());
        }
    }

    // we trust the feeds since
    //
    // - they are peers that we've connected to (validators)
    // - the best head is at least a descendant of their finalized heads
    //
    // so we can directly fetch the missing blocks from the feeds.
    feeds.sort_by_key(|conn| conn.latency);
    let Some(feed) = feeds.first() else {
        tracing::warn!(
            "no peers found for syncing to the best head block#{}@{}",
            best.slot,
            hex::encode(best.hash)
        );
        return Ok(());
    };

    // send the request to the feed.
    let mut recv = ce128::send(
        feed.clone(),
        ce128::Request {
            hash: grandpa.head.hash,
            direction: 0,
            maximum: best.slot.saturating_sub(grandpa.head.slot),
        },
    )
    .await?;

    // receive the blocks from the feed.
    let mut buffer = Vec::new();
    while let Some(chunk) = recv.read_chunk(1, true).await? {
        buffer.extend_from_slice(&chunk.bytes);
        let Ok(blocks) = codec::decode::<Vec<Block>>(&buffer) else {
            continue;
        };

        for block in blocks {
            runtime.finalize(&block).await?;
            tracing::info!(
                "finalized block#{}@0x{}",
                block.header.slot,
                hex::encode(block.hash()?)
            );
        }
    }

    Ok(())
}
