//! Importer interface for the chain.

use crate::{
    chain::fork::{BlockWithDiff, Fork},
    storage::{Branch, KVStorage, SyncStorage},
    Chain, Config,
};
use anyhow::Result;
use score::{
    block::{Head, Header},
    state::key,
    Block,
};
use std::collections::HashMap;

impl<C: Config> Chain<C> {
    /// Get the best fork of the chain.
    pub fn best(&self) -> Result<Head> {
        // if there are no forks, returns the finalized head directly.
        if self.forks.is_empty() {
            return Ok(self.grandpa.handshake.head.clone());
        }

        // find the best chain
        let mut count = 0;
        let mut best = None;
        for (_, fork) in self.forks.iter() {
            let flen = fork.len();
            if flen > count {
                count = flen;
                best = Some(fork.best()?);
            }
        }

        // if there is no best chain, returns an error
        best.ok_or_else(|| anyhow::anyhow!("could not find the best head"))
    }

    /// Try finalize the chain.
    pub fn finalize(&mut self) -> Result<Vec<BlockWithDiff>> {
        let best = self.best()?;
        if best.hash == self.grandpa.handshake.head.hash {
            return Ok(vec![]);
        }

        // find the best chain
        let Some(mut chain) = self.forks.get(&best.hash).cloned() else {
            anyhow::bail!("could not find the best chain");
        };

        // nothing to finalize if the best fork chain is less than 5 blocks.
        let Some(count) = chain.len().checked_sub(5) else {
            return Ok(vec![]);
        };

        // truncate the series.
        let timeslot = self.grandpa.handshake.head.slot;
        let latest = timeslot + count as u32;
        let epoch = latest / score::EPOCH_LENGTH;
        chain.series.retain(|k, _| k < &epoch);

        // apply the latest finalized blocks.
        let mut blocks = Vec::new();
        while let Some((slot, (block, commit))) = chain.blocks.pop_first() {
            let hash = block.header.hash()?;
            blocks.push((block, commit.clone()));
            self.state.commit_legacy(commit)?;
            tracing::info!("finalized block#{}@0x{}", slot, hex::encode(&hash[..3]));

            if slot == latest {
                break;
            }
        }

        // now we need to truncate all fork chains.
        self.forks.retain(|head, _fork| {
            if !chain.chain.iter().any(|h| h.hash == *head) {
                return false;
            }

            true
        });
        self.forks.insert(chain.head()?.hash, chain);
        Ok(blocks)
    }

    /// Create a new fork at the latest finalized block.
    pub fn fork(&mut self, block: &Block) -> Result<()> {
        let hash = block.header.hash()?;
        let branch = Branch::checkout(self.state.clone());
        let mut fork = Fork::new(branch, self.grid.clone(), self.series.clone());
        fork.import::<C::Vm>(block)?;
        self.forks.insert(hash, fork);
        Ok(())
    }

    /// Import a new block to the chain.
    pub async fn import(&mut self, block: &Block) -> anyhow::Result<()> {
        let head = block.header.head()?;
        if block.header.slot <= self.grandpa.handshake.head.slot {
            tracing::trace!(
                "Discarding block#{}@0x{}... since it's before the finalized block={}",
                head.slot,
                hex::encode(&head.hash[..6]),
                self.grandpa.handshake.head.slot
            );
            return Ok(());
        }

        // 1. the block is a child of the finalized
        if block.header.parent == self.grandpa.handshake.head.hash {
            return self.fork(block);
        }

        // 2. the block is a child of a fork
        for (_, fork) in self.forks.iter_mut() {
            if fork.best()?.hash == block.header.parent {
                return fork.import::<C::Vm>(block);
            }

            if fork.chain.iter().any(|h| h.hash == block.header.parent) {
                let fork = fork.fork::<C::Vm>(block)?;
                self.forks.insert(head.hash, fork);
                return Ok(());
            }
        }

        // 3. we don't have the ancestors of this block
        self.orphan.insert(head.hash, block.clone());
        Ok(())
    }

    /// Import the genesis block
    pub async fn import_genesis(
        &mut self,
        header: Header,
        state: &HashMap<[u8; 31], Vec<u8>>,
    ) -> anyhow::Result<()> {
        // 1. save the block to the storage
        let head = header.head()?;
        self.state.finalize(&head)?;

        // 2. set the genesis state
        let mut kvs = Vec::new();
        for (key, value) in state {
            kvs.push((key.to_vec(), value.clone()));
            match *key {
                key::PREVIOUS_VALIDATORS => {
                    self.grid.prev = codec::decode(value)?;
                }
                key::CURRENT_VALIDATORS => {
                    self.grid.curr = codec::decode(value)?;
                }
                key::NEXT_VALIDATORS => {
                    self.grid.next = codec::decode(value)?;
                }
                _ => {}
            }
        }

        self.state.commit((kvs, vec![]).into())?;
        self.grandpa.handshake.head = head;
        Ok(())
    }
}
