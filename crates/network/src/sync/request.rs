//! Block sync requester

use crate::{
    stream::{ce128, ext::Read},
    Connection, Network,
};
use runtime::{storage::SyncStorage, Ancestry};
use score::Block;

/// Block sync requester
pub struct BlockSync<'r, C: runtime::Config> {
    /// The ancestry of the sync.
    ancestry: Ancestry,

    /// The runtime of the sync.
    runtime: &'r Network<C>,
}

impl<'r, C: runtime::Config> BlockSync<'r, C> {
    /// Create a new block sync requester.
    ///
    /// TODO: indicate the request direction by the maximum blocks.
    pub async fn asc(runtime: &'r Network<C>, ancestry: Ancestry) -> anyhow::Result<Self> {
        tracing::debug!(
            "syncing from local#{}@0x{} to best#{}@0x{} ({} blocks)",
            ancestry.finalized.slot,
            hex::encode(&ancestry.finalized.hash[..3]),
            ancestry.best.slot,
            hex::encode(&ancestry.best.hash[..3]),
            ancestry.best.slot - ancestry.finalized.slot
        );

        Ok(Self { ancestry, runtime })
    }

    /// Send the request to the feeds.
    #[tracing::instrument(skip_all, parent = None, name = "sync::remote")]
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        if self.ancestry.finalized.slot == self.ancestry.best.slot {
            return Ok(());
        }

        let feeds = self.runtime.lookup(&self.ancestry.best).await;
        for feed in feeds {
            if let Err(e) = self.request(&feed).await {
                tracing::debug!(
                    "failed to sync from {}: {}, switching to the next feed",
                    feed.address.peer_id,
                    e
                );

                continue;
            }

            break;
        }

        Ok(())
    }

    /// Send the request to the feeds.
    ///
    /// NOTE: seems polkajam doesn't support multiple blocks atm.
    pub async fn request(&mut self, feed: &Connection) -> anyhow::Result<()> {
        let best = self.ancestry.best.hash;
        let mut head = self.ancestry.finalized.hash;
        loop {
            let mut recv = ce128::send(
                feed,
                ce128::Request {
                    hash: head,
                    direction: 0,
                    maximum: 1,
                },
            )
            .await?;

            let block: Block = Block::read(&mut recv).await?;
            let hash = block.header.hash()?;
            tracing::trace!(
                "received block#{}@0x{}",
                block.header.slot,
                hex::encode(&hash[..3])
            );

            self.runtime.announce(block.header.clone()).await?;
            if let Err(e) = self.runtime.import(block.clone()).await {
                tracing::warn!(
                    "failed to import block#{}@0x{}: {e}",
                    block.header.slot,
                    hex::encode(&hash[..3])
                );

                self.runtime.storage.set_block(&block)?;
                return Ok(());
            }

            head = hash;
            if head == best {
                break;
            }
        }

        Ok(())
    }
}
