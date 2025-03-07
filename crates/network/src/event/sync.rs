//! Handler of sync events

use crate::{stream::ce128, Network};
use score::{block::Header, runtime::Head, Block, TimeSlot};

/// Announce a block to the network
pub async fn announce<C: score::runtime::Config>(
    runtime: Network<C>,
    header: Box<Header>,
    head: Head,
) -> anyhow::Result<()> {
    if let Err(e) = runtime.grandpa.read().await.verify(&header).await {
        anyhow::bail!(e);
    }

    // broadcast the block to the network
    runtime.announce.send((*header, head))?;
    Ok(())
}

/// Select the best chain.
///
/// This happens on:
/// - receving new block announcements
/// - before authoring blocks
pub async fn select_best_chain<C: score::runtime::Config>(
    runtime: Network<C>,
    slot: TimeSlot,
) -> anyhow::Result<()> {
    let grandpa = runtime.grandpa.read().await;
    if slot <= grandpa.head.slot {
        return Ok(());
    }

    // select the best head from the grandpa.
    let Some(best) = grandpa.select_best_head() else {
        return Ok(());
    };

    let mut feeds = Vec::new();
    for conn in runtime.pool.read().await.values() {
        let head = conn.handshake.read().await.head.clone();
        if head.hash == best.hash
            || head.slot > best.slot && grandpa.is_descendant_of(&head.hash, best.hash)
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
    if let Some(blocks) = recv.read_chunk(10, true).await? {
        let blocks: Vec<Block> = codec::decode(&blocks.bytes)?;
        for block in blocks {
            runtime.finalize(&block).await?;
        }
    }

    Ok(())
}
