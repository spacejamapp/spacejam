//! Sync implementation of Spacejam.

use crate::Network;
use request::BlockSync;
use runtime::{
    storage::{ArchiveStorage, SyncStorage},
    Hook, Storage,
};
use score::{block::Head, OpaqueHash, TimeSlot};

mod request;

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
        let best = self.storage.best()?;

        if best.slot == slot {
            tracing::trace!(
                "skipping best chain selection: incoming#{}, grandpa#{}",
                slot,
                best.slot
            );
            return Ok(());
        } else if best.slot > slot {
            tracing::warn!(
                "TODO: best head={} is ahead of the incoming timeslot={slot}",
                best.slot
            );
        }

        // select the best head from the grandpa.
        let Some(mut ancestry) = grandpa.select_best_head() else {
            return Ok(());
        };

        tracing::trace!(
            "head#{}@0x{} ancestors: {:#?}",
            ancestry.best.slot,
            hex::encode(&ancestry.best.hash[..3]),
            ancestry
                .ancestors
                .iter()
                .map(|h| format!("0x{}", hex::encode(h)))
                .collect::<Vec<_>>()
        );

        if ancestry.ancestors.len() > 3 {
            self.finalize(&ancestry.ancestors[3..]).await?;
        }

        // WORKAROUND: fallback to the finalized head.
        if let Err(e) = ancestry.advance(&best) {
            self.fallback().await?;
            return Err(e);
        }

        BlockSync::asc(self, ancestry).await?.sync().await?;
        Ok(())
    }

    /// Fallback to the finalized chain.
    ///
    /// This happens when our best head is on a fork chain
    ///
    /// TODO: this operation should be well tested.
    pub async fn fallback(&self) -> anyhow::Result<()> {
        let finalized = self.storage.finalized()?;
        self.storage.finalize(finalized.hash)?;
        tracing::warn!(
            "fallback to the finalized chain at head#{}@0x{}",
            finalized.slot,
            hex::encode(finalized.hash)
        );

        self.grandpa.write().await.handshake.head = finalized;
        Ok(())
    }
}
