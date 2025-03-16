//! Handler of sync events

use crate::{stream::ce128, Network};
use quinn::RecvStream;
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
    name = "finalize",
    fields(slot = ?slot)
)]
pub async fn select_best_chain<C: score::runtime::Config>(
    runtime: Network<C>,
    slot: TimeSlot,
) -> anyhow::Result<()> {
    tracing::debug!("selecting the best chain");
    let grandpa = runtime.grandpa.read().await.clone();
    if slot <= grandpa.handshake.head.slot {
        tracing::trace!("skipping select best chain because of duplicated slot");
        return Ok(());
    }

    // select the best head from the grandpa.
    let Some((best, ancestors)) = grandpa.select_best_head() else {
        tracing::warn!("failed to select the best head");
        return Ok(());
    };

    // if the best head is already in the local storage,
    // run sync from the local storage.
    let chain = runtime.chain().await;
    if let Ok(head) = chain.get_block(&best.hash) {
        self::finalize_local(&runtime, head, ancestors).await
    } else {
        BlockSync::new(&runtime, best).await.sync().await
    }
}

/// Finalize blocks from the local chain.
#[tracing::instrument(skip_all, name = "local")]
async fn finalize_local<C: score::runtime::Config>(
    runtime: &Network<C>,
    head: Block,
    mut ancestors: Vec<(OpaqueHash, Header)>,
) -> anyhow::Result<()> {
    ancestors.reverse();
    let grandpa = runtime.grandpa.read().await.clone();
    let chain = runtime.chain().await;
    let mut current = grandpa.handshake.head.clone();
    let importer = runtime.importer();
    for (ancestor, header) in ancestors.iter().skip(1) {
        if header.parent != current.hash {
            anyhow::bail!(
                "ancestor {} is not the parent of {}",
                hex::encode(ancestor),
                hex::encode(current.hash)
            );
        }

        importer.finalize(chain.get_block(ancestor)?).await?;
        current = Head {
            hash: *ancestor,
            slot: header.slot,
        };
    }

    importer.finalize(head).await?;
    Ok(())
}

/// An block sync requester.
pub struct BlockSync<'r, C: score::runtime::Config> {
    /// The best head of the sync.
    best: Head,

    /// The current state of the request.
    request: ce128::Request,

    /// The runtime of the sync.
    runtime: &'r Network<C>,
}

impl<'r, C: score::runtime::Config> BlockSync<'r, C> {
    /// Create a new block sync requester.
    pub async fn new(runtime: &'r Network<C>, best: Head) -> Self {
        let grandpa = runtime.grandpa.read().await.clone();
        let request = ce128::Request {
            hash: best.hash,
            direction: 0,
            maximum: (grandpa
                .ancestors(&best.hash, grandpa.handshake.head.hash)
                .len() as u32)
                + 1,
        };

        Self {
            best,
            request,
            runtime,
        }
    }

    /// Send the request to the feeds.
    #[tracing::instrument(skip_all, name = "remote")]
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let feeds = self.runtime.lookup(&self.best).await;
        for feed in feeds {
            if self.request.maximum == 0 {
                break;
            }

            tracing::info!(
                "request {} for block#{}@0x{} with maximum {} blocks",
                feed.address.peer_id.to_string(),
                self.best.slot,
                hex::encode(&self.best.hash[..3]),
                self.request.maximum,
            );

            let (mut send, recv) = ce128::send(feed.clone(), self.request.clone()).await?;
            if let Err(e) = self.request(recv).await {
                tracing::warn!("failed to request from {}: {}", feed.address.peer_id, e);
            }

            send.finish()?;
            continue;
        }

        Ok(())
    }

    /// Send the request to the feeds.
    pub async fn request(&mut self, mut recv: RecvStream) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        let importer = self.runtime.importer();
        while let Some(chunk) = recv.read_chunk(1, true).await? {
            buffer.extend_from_slice(&chunk.bytes);
            let Ok(block) = codec::decode::<Block>(&buffer) else {
                continue;
            };

            buffer.clear();
            tracing::debug!(
                "received block#{}@{}",
                block.header.slot,
                hex::encode(&block.header.hash()?[..3])
            );
            let grandpa = self.runtime.grandpa.read().await.clone();
            if grandpa.handshake.head.slot >= block.header.slot {
                continue;
            }

            // finalize the block.
            let head: Head = block.header.clone().try_into()?;
            importer.finalize(block).await?;

            // update the request.
            self.request.maximum = self.request.maximum.saturating_sub(1);
            self.request.hash = head.hash;
            self.best = head;
        }
        Ok(())
    }
}
