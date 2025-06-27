use crate::{stream::ce128, Network};
use quinn::RecvStream;
use runtime::storage::SyncStorage;
use score::{block::Head, Block, OpaqueHash};

/// Block sync requester
pub struct BlockSync<'r, C: runtime::Config> {
    /// The best head of the sync.
    target: Head,

    /// The current state of the request.
    request: ce128::Request,

    /// The runtime of the sync.
    runtime: &'r Network<C>,
}

impl<'r, C: runtime::Config> BlockSync<'r, C> {
    /// Create a new block sync requester.
    ///
    /// TODO: indicate the request direction by the maximum blocks.
    pub async fn asc(
        runtime: &'r Network<C>,
        start: OpaqueHash,
        target: Head,
        to_request: usize,
    ) -> anyhow::Result<Self> {
        tracing::debug!(
            "syncing from 0x{} to 0x{} ({} blocks)",
            hex::encode(&start[..3]),
            hex::encode(&target.hash[..3]),
            to_request
        );
        let request = ce128::Request {
            hash: start,
            direction: 0,
            maximum: to_request as u32,
        };

        Ok(Self {
            target,
            request,
            runtime,
        })
    }

    /// Send the request to the feeds.
    #[tracing::instrument(skip_all, parent = None, name = "sync::remote")]
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let feeds = self.runtime.lookup(&self.target).await;
        for feed in feeds {
            if self.request.maximum == 0 {
                return Ok(());
            }

            let recv = ce128::send(feed.clone(), self.request.clone()).await?;
            if let Err(e) = self.request(recv).await {
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
    pub async fn request(&mut self, mut recv: RecvStream) -> anyhow::Result<()> {
        let mut requested = 0;
        loop {
            let mut buf = [0; 4];
            if recv.read_exact(&mut buf).await.is_err() {
                // NOTE: seems polkajam doesn't support multiple blocks atm.
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
