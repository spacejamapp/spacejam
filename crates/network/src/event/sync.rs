//! Handler of sync events

use crate::{stream::ce128, Network};
use score::{
    block::Header,
    runtime::{storage::BlockStorage, Head},
    Block, OpaqueHash, TimeSlot,
};

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
    if slot <= grandpa.handshake.head.slot {
        return Ok(());
    }

    // select the best head from the grandpa.
    let Some((best, ancestors)) = grandpa.select_best_head() else {
        return Ok(());
    };

    // if the best head is already in the local storage,
    // run sync from the local storage.
    let chain = runtime.chain().await;
    if let Ok(head) = chain.get_block(&best.hash) {
        self::finalize_locally(&runtime, head, ancestors).await
    } else {
        self::finalize_from_feed(&runtime, best).await
    }
}

/// Finalize blocks from the local chain.
#[tracing::instrument(skip_all, level = "debug")]
async fn finalize_locally<C: score::runtime::Config>(
    runtime: &Network<C>,
    head: Block,
    mut ancestors: Vec<(OpaqueHash, Header)>,
) -> anyhow::Result<()> {
    tracing::debug!("finalizing from local chain ...");
    ancestors.reverse();
    let grandpa = runtime.grandpa.read().await.clone();
    let chain = runtime.chain().await;
    let mut current = grandpa.handshake.head.clone();
    for (ancestor, header) in ancestors.iter().skip(1) {
        if header.parent != current.hash {
            anyhow::bail!(
                "ancestor {} is not the parent of {}",
                hex::encode(ancestor),
                hex::encode(current.hash)
            );
        }

        runtime.finalize(&chain.get_block(ancestor)?).await?;
        current = Head {
            hash: *ancestor,
            slot: header.slot,
        };
    }

    runtime.finalize(&head).await?;
    Ok(())
}

/// Finalize blocks from the feed.
async fn finalize_from_feed<C: score::runtime::Config>(
    runtime: &Network<C>,
    best: Head,
) -> anyhow::Result<()> {
    let grandpa = runtime.grandpa.read().await.clone();
    let Some(feed) = runtime.lookup(&best).await else {
        return Ok(());
    };

    // send the request to the feed.
    tracing::debug!("finalizing from feed .");
    let request = ce128::Request {
        hash: best.hash,
        direction: 0,
        maximum: best.slot.saturating_sub(grandpa.handshake.head.slot),
    };
    let (mut send, mut recv) = ce128::send(feed.clone(), request.clone()).await?;

    // receive the blocks from the feed.
    tracing::trace!(
        "request for {} blocks with maximum {} blocks",
        hex::encode(request.hash),
        request.maximum,
    );
    let mut buffer = Vec::new();
    while let Some(chunk) = recv.read_chunk(1, true).await? {
        buffer.extend_from_slice(&chunk.bytes);
        let Ok(block) = codec::decode::<Block>(&buffer) else {
            continue;
        };

        buffer.clear();
        tracing::debug!("received block#{}", block.header.slot);
        let grandpa = runtime.grandpa.read().await.clone();
        if grandpa.handshake.head.slot >= block.header.slot {
            continue;
        }

        runtime.finalize(&block).await?;
    }

    send.finish()?;
    Ok(())
}
