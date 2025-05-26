//! Handler of sync events

use crate::{stream::ce128, Network};
use quinn::RecvStream;
use runtime::storage::SyncStorage;
use score::{
    block::{Head, Header},
    Block, OpaqueHash, TimeSlot,
};

impl<C: runtime::Config> Network<C> {
    /// Select the best chain.
    ///
    /// This happens on:
    /// - receiving new block announcements
    /// - before authoring blocks
    #[tracing::instrument(
    skip_all,
    name = "finalize",
    parent = None,
    fields(slot = ?slot)
)]
    /// Select the best chain.
    pub async fn select_best_chain(&self, slot: TimeSlot) -> anyhow::Result<()> {
        let grandpa = self.grandpa.read().await.clone();
        if slot <= grandpa.handshake.head.slot {
            tracing::trace!(
                "upcoming#{slot}, grandpa#{}: skipping best chain selection",
                grandpa.handshake.head.slot
            );
            return Ok(());
        }

        // select the best head from the grandpa.
        let Some((best, ancestors)) = grandpa.select_best_head() else {
            return Ok(());
        };

        // if the best head is already in the local storage,
        // run sync from the local storage.
        if let Ok(head) = self.storage.get_block(&best.hash) {
            self.finalize_local(head, ancestors).await
        } else {
            BlockSync::new(self, best).await?.sync().await
        }
    }

    /// Finalize blocks from the local chain.
    #[tracing::instrument(skip_all, name = "local")]
    async fn finalize_local(
        &self,
        best: Block,
        mut ancestors: Vec<(OpaqueHash, Header)>,
    ) -> anyhow::Result<()> {
        ancestors.reverse();
        let grandpa = self.grandpa.read().await.clone();
        let mut finalized = grandpa.handshake.head.clone();
        for (ancestor, header) in ancestors.iter() {
            if header.slot == best.header.slot {
                break;
            }

            if header.parent != finalized.hash {
                anyhow::bail!(
                "the parent 0x{} of the ancestor#{}@0x{} is not the latest finalized block#{}@0x{}",
                hex::encode(&header.parent[..3]),
                header.slot,
                hex::encode(&ancestor[..3]),
                finalized.slot,
                hex::encode(&finalized.hash[..3]),
            );
            }

            self.finalize(self.storage.get_block(ancestor)?).await?;
            finalized = Head {
                hash: *ancestor,
                slot: header.slot,
            };
        }

        self.finalize(best).await?;
        Ok(())
    }
}

/// An block sync requester.
pub struct BlockSync<'r, C: runtime::Config> {
    /// The best head of the sync.
    best: Head,

    /// The current state of the request.
    request: ce128::Request,

    /// The runtime of the sync.
    runtime: &'r Network<C>,
}

impl<'r, C: runtime::Config> BlockSync<'r, C> {
    /// Create a new block sync requester.
    ///
    /// TODO: indicate the request direction by the maximum blocks.
    pub async fn new(runtime: &'r Network<C>, best: Head) -> anyhow::Result<Self> {
        let grandpa = runtime.grandpa.read().await.clone();
        let request = ce128::Request {
            hash: grandpa.handshake.head.hash,
            direction: 0,
            maximum: (grandpa
                .ancestors(&best.hash, grandpa.handshake.head.hash)
                .len() as u32)
                + 1,
        };

        Ok(Self {
            best,
            request,
            runtime,
        })
    }

    /// Send the request to the feeds.
    #[tracing::instrument(skip_all, name = "remote")]
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let feeds = self.runtime.lookup(&self.best).await;
        for feed in feeds {
            if self.request.maximum == 0 {
                break;
            }

            tracing::trace!(
                "request {} for block#{}@0x{} with maximum {} blocks",
                feed.address.peer_id.to_string(),
                self.best.slot,
                hex::encode(&self.best.hash[..3]),
                self.request.maximum,
            );

            let recv = ce128::send(feed.clone(), self.request.clone()).await?;
            if let Err(e) = self.request(recv).await {
                tracing::debug!(
                    "failed to sync from {}: {}, swithing to the next feed",
                    feed.address.peer_id,
                    e
                );
                continue;
            }
        }

        Ok(())
    }

    /// Send the request to the feeds.
    ///
    /// TODO: If our local storage contains one of the upcoming blocks,
    /// we should finalize it from our storage directly.
    pub async fn request(&mut self, mut recv: RecvStream) -> anyhow::Result<()> {
        let mut buf = [0; 4];
        recv.read_exact(&mut buf).await?;
        let length = u32::from_le_bytes(buf);

        let mut buffer = vec![0; length as usize];
        recv.read_exact(&mut buffer).await?;
        let block: Block = codec::decode(&buffer)?;
        let blocks = vec![block];

        for block in blocks {
            // if the block is considered as a descendant of the current head, skip it.
            {
                let grandpa = self.runtime.grandpa.read().await.clone();
                if block.header.hash()? == grandpa.handshake.head.hash {
                    continue;
                }

                if grandpa.handshake.head.slot >= block.header.slot {
                    continue;
                }
            }

            // finalize the block.
            self.runtime.finalize(block).await?;
        }

        Ok(())
    }
}
