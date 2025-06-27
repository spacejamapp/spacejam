//! Handler of sync events

use crate::{stream::ce128, Network};
use quinn::RecvStream;
use runtime::{
    storage::{ArchiveStorage, SyncStorage},
    Hook, Storage,
};
use score::{block::Head, Block, OpaqueHash, TimeSlot};

impl<C: runtime::Config> Network<C> {
    /// Finalize blocks from the local chain.
    #[tracing::instrument(skip_all, name = "finalize")]
    async fn finalize(&self, ancestors: &[OpaqueHash]) -> anyhow::Result<()> {
        let mut ancestors = ancestors.to_vec();
        ancestors.reverse();
        let grandpa = self.grandpa().await;
        let mut finalized = grandpa.handshake.head.clone();
        let mut block = Default::default();
        for head in ancestors.iter() {
            // FIXME: using block instead of header here since we may not have header
            // of requested block series??
            let Ok(ancestor) = self.storage.block(head) else {
                break;
            };
            block = ancestor;
            let header = block.header.clone();
            let slot = header.slot;

            if header.parent != finalized.hash {
                anyhow::bail!(
                "the parent 0x{} of the ancestor#{}@0x{} is not the latest finalized block#{}@0x{}",
                hex::encode(&header.parent[..3]),
                header.slot,
                hex::encode(&head[..3]),
                finalized.slot,
                hex::encode(&finalized.hash[..3]),
            );
            }

            tracing::info!("block#{}@0x{}", slot, hex::encode(&head[..3]));
            finalized = Head { hash: *head, slot };
        }

        self.storage.finalize(finalized.hash)?;
        self.storage.set_finalized(&finalized)?;
        {
            // FIXME: this could introduce bugs in future.
            let next = if block.header.epoch_mark.is_some() {
                Some(self.storage.next_validators()?)
            } else {
                None
            };

            self.grandpa
                .write()
                .await
                .finalize(block.header.clone(), next)?;
        }

        let diff = self.storage.diff(finalized.hash)?;
        self.hook.on_finalized_block(block).await?;
        self.hook.on_diff(finalized.hash, diff).await?;
        self.grandpa.write().await.handshake.head = finalized;
        Ok(())
    }

    /// Select the best chain.
    ///
    /// This happens on:
    /// - receiving new block announcements
    /// - before authoring blocks
    #[tracing::instrument(skip_all, name = "select", parent = None)]
    pub async fn select_best_chain(&self, slot: TimeSlot) -> anyhow::Result<()> {
        let grandpa = self.grandpa().await;
        let mut best = self.storage.best()?;

        if best.slot == slot {
            tracing::trace!(
                "skipping best chain selection: incoming#{}, grandpa#{}",
                slot,
                best.slot
            );
            return Ok(());
        } else if best.slot > slot {
            let finalized = self.storage.finalized()?;
            if self
                .storage
                .header(&finalized.hash)
                .and_then(|h| self.storage.block(&h.hash()?))
                .is_err()
            {
                tracing::warn!(
                    "switching best chain@{} ...",
                    hex::encode(&finalized.hash[..3])
                );
                self.storage.finalize(finalized.hash)?;
                self.storage.set_best(&finalized)?;
                best = finalized;
            }
        }

        // select the best head from the grandpa.
        let Some((target, mut ancestors)) = grandpa.select_best_head() else {
            return Ok(());
        };

        tracing::info!(
            "head#{}@0x{} ancestors: {:#?}",
            target.slot,
            hex::encode(&target.hash[..3]),
            ancestors
                .iter()
                .map(|h| format!("0x{}", hex::encode(h)))
                .collect::<Vec<_>>()
        );
        if ancestors.len() > 3 {
            self.finalize(&ancestors[3..]).await?;
        }

        // if the best head is already in the local storage,
        // run sync from the local storage.
        if self.storage.block(&target.hash).is_err() {
            // Try import missing blocks directly
            {
                let mut imported = 0;
                for hash in ancestors.iter().rev() {
                    let Ok(block) = self.storage.block(hash) else {
                        break;
                    };

                    if block.header.slot <= best.slot {
                        imported += 1;
                        continue;
                    }

                    best = Head {
                        hash: *hash,
                        slot: block.header.slot,
                    };

                    // TODO: pick the diff instead of re-executing it.
                    self.runtime.import(block).await?;
                    imported += 1;
                }

                if imported > 0 {
                    ancestors.truncate(ancestors.len() - imported);
                }
            }

            BlockSync::asc(self, best.hash, target, ancestors.len() + 1)
                .await?
                .sync()
                .await
        } else {
            Ok(())
        }
    }
}

/// An block sync requester.
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
    #[tracing::instrument(skip_all, parent = None, name = "sync")]
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
