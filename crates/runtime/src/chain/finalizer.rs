//! Finalizer interface for the chain.

use crate::{
    chain::fork::BlockWithDiff,
    storage::{Column, KVStorage, StateStorage, SyncStorage},
    Chain, Config,
};
use anyhow::Result;
use score::block::Head;
use std::collections::BTreeSet;

impl<C: Config> Chain<C> {
    /// Try finalize the chain.
    pub fn finalize(&mut self) -> Result<Vec<BlockWithDiff>> {
        tracing::trace!("finalizing the chain...");
        self.resolve_orphan()?;
        let best = self.best()?;
        tracing::trace!("best: #{}@0x{}", best.slot, hex::encode(&best.hash[..3]));
        if best.hash == self.grandpa.handshake.head.hash {
            tracing::trace!("best is the same as the finalized head");
            return Ok(vec![]);
        }

        // find the best chain
        let Ok(mut chain) = self.best_chain() else {
            tracing::trace!("no fork chain found for best");
            return Ok(vec![]);
        };

        // nothing to finalize if the best fork chain is less than 5 blocks.
        tracing::trace!("best chain length: {}", chain.len());
        let Some(count) = chain.len().checked_sub(13) else {
            return Ok(vec![]);
        };

        // truncate the series.
        let timeslot = self.grandpa.handshake.head.slot;
        let latest = timeslot + count as u32;
        self.series = chain.series.clone();
        self.grid = chain.grid.clone();

        tracing::trace!("updated chain series ...");
        // apply the latest finalized blocks.
        let mut blocks = Vec::new();
        let mut finalized = BTreeSet::new();
        while let Some((slot, (block, commit))) = chain.blocks.pop_first() {
            let head = block.header.head()?;
            self.state.commit(Column::State, commit.clone())?;

            // finalize the block in storage
            let root = self.state.root()?;
            self.state.finalize(&block, head.hash, root)?;
            tracing::info!(
                "finalized block#{}@0x{}",
                slot,
                hex::encode(&head.hash[..3])
            );

            self.grandpa.handshake.head = head.clone();
            blocks.push((block, commit));
            finalized.insert(head);
            if slot >= latest {
                break;
            }
        }

        // reset forks after finalization
        self.reset_forks(finalized)?;

        // handle orphan blocks
        self.process_orphan()?;
        Ok(blocks)
    }

    fn reset_forks(&mut self, finalized: BTreeSet<Head>) -> Result<()> {
        // Reset forks after finalization
        let head = &self.grandpa.handshake.head;
        if !finalized.is_empty() {
            let finalized_slots = finalized.iter().map(|h| h.slot).collect::<BTreeSet<_>>();
            let forks = std::mem::take(&mut self.forks);

            // Rebuild forks with only valid ones
            for (_, mut fork) in forks {
                fork.chain.retain(|h| !finalized.contains(h));
                fork.blocks
                    .retain(|slot, _| !finalized_slots.contains(slot));
                if fork.chain.is_empty() {
                    continue;
                }

                // Skip forks where the best block is older than the finalized head
                if let Ok(best) = fork.best() {
                    if best.slot < head.slot {
                        continue;
                    }

                    self.forks.insert(fork.head()?.hash, fork);
                }
            }

            tracing::trace!("{} forks remaining after finalization", self.forks.len());
        }
        Ok(())
    }

    /// try process the orphan blocks
    fn process_orphan(&mut self) -> Result<()> {
        let finalized = &self.grandpa.handshake.head;
        let mut to_remove = Vec::new();
        let mut to_import = Vec::new();
        for (slot, blocks) in self.orphan.iter() {
            for (hash, block) in blocks.iter() {
                if block.header.slot <= finalized.slot {
                    to_remove.push((*slot, *hash));
                }

                to_import.push((*slot, *hash, block.clone()));
            }
        }

        for (slot, hash, block) in to_import.into_iter() {
            if self.import(&block)?.imported() {
                to_remove.push((slot, hash));
            }
        }

        for (slot, hash) in &to_remove {
            self.orphan.entry(*slot).or_default().remove(hash);
        }

        Ok(())
    }
}
