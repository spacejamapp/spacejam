//! Block sync requester

use crate::{stream::ce128, Connection, Network};
use runtime::{storage::SyncStorage, Ancestry};
use score::Block;

/// Block sync requester
pub struct BlockSync<'r, C: runtime::Config> {
    /// The ancestry of the sync.
    ancestry: Ancestry,

    /// The current state of the request.
    request: ce128::Request,

    /// The runtime of the sync.
    runtime: &'r Network<C>,
}

impl<'r, C: runtime::Config> BlockSync<'r, C> {
    /// Create a new block sync requester.
    ///
    /// TODO: indicate the request direction by the maximum blocks.
    pub async fn asc(runtime: &'r Network<C>, ancestry: Ancestry) -> anyhow::Result<Self> {
        tracing::debug!(
            "syncing from 0x{} to 0x{} ({} blocks)",
            hex::encode(&ancestry.finalized.hash[..3]),
            hex::encode(&ancestry.best.hash[..3]),
            ancestry.ancestors.len() + 1
        );
        let request = ce128::Request {
            hash: ancestry.best.hash,
            direction: 0,
            maximum: ancestry.ancestors.len() as u32 + 1,
        };

        Ok(Self {
            ancestry,
            request,
            runtime,
        })
    }

    /// Send the request to the feeds.
    #[tracing::instrument(skip_all, parent = None, name = "sync::remote")]
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let feeds = self.runtime.lookup(&self.ancestry.best).await;
        for feed in feeds {
            if self.request.maximum == 0 {
                return Ok(());
            }

            if let Err(e) = self.request_all(&feed).await {
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
    pub async fn request_all(&mut self, feed: &Connection) -> anyhow::Result<()> {
        let mut recv = ce128::send(feed, self.request.clone()).await?;
        let mut requested = 0;
        loop {
            let mut buf = [0; 4];
            if recv.read_exact(&mut buf).await.is_err() {
                tracing::warn!("no more blocks to read, FIXME: read multiple blocks");
                break;
            }

            let length = u32::from_le_bytes(buf);
            let mut buffer = vec![0; length as usize];
            if recv.read_exact(&mut buffer).await.is_err() {
                break;
            }

            let block: Block = codec::decode(&buffer)?;
            if self.runtime.storage.block(&block.header.hash()?).is_err() {
                // TODO: set instead of import if the block is not consistent
                // with the best head.
                self.runtime.import(block).await?;
            }

            requested += 1;
            if requested >= self.request.maximum {
                break;
            }
        }

        Ok(())
    }
}
