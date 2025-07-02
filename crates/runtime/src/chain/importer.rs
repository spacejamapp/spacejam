//! Importer interface for the chain.

use crate::{
    chain::fork::{BlockWithDiff, Fork},
    storage::{Branch, Column, KVStorage, StateStorage, SyncStorage},
    Chain, Config,
};
use anyhow::{Context, Result};
use score::{
    block::{Head, Header},
    state::key,
    Block,
};
use std::{collections::HashMap, sync::Arc};

impl<C: Config> Chain<C> {
    /// Get the best head of the chain.
    pub fn best(&self) -> Result<Head> {
        // if there are no forks, returns the finalized head directly.
        if self.forks.is_empty() {
            return Ok(self.grandpa.handshake.head.clone());
        }

        self.best_chain()
            .context("forks are not empty, but no best chain found")?
            .best()
    }

    /// Get the best chain
    pub fn best_chain(&self) -> Result<&Fork<C::Storage>> {
        let mut count = 0;
        let mut best = None;
        for (hash, fork) in self.forks.iter() {
            let flen = fork.len();
            if flen > count {
                count = flen;
                best = Some(hash);
            }
        }

        best.and_then(|hash| self.forks.get(hash))
            .ok_or_else(|| anyhow::anyhow!("could not find the best chain"))
    }

    /// Try finalize the chain.
    pub fn finalize(&mut self) -> Result<Vec<BlockWithDiff>> {
        tracing::trace!("finalizing the chain...");
        let best = self.best()?;
        if best.hash == self.grandpa.handshake.head.hash {
            return Ok(vec![]);
        }

        // find the best chain
        let Some(mut chain) = self.forks.get(&best.hash).cloned() else {
            return Ok(vec![]);
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
        self.series = chain.series.clone();
        self.grid = chain.grid.clone();

        // apply the latest finalized blocks.
        let mut blocks = Vec::new();
        while let Some((slot, (block, commit))) = chain.blocks.pop_first() {
            let hash = block.header.hash()?;
            self.state.commit(Column::State, commit.clone())?;
            let root = self.state.root()?;
            self.state.finalize(&block, hash, root)?;
            self.grandpa.handshake.head = Head { slot, hash };
            blocks.push((block, commit));
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
        let head = self.grandpa.handshake.head.clone();
        let hash = block.header.hash()?;
        let branch = Branch::checkout(self.state.clone());
        let mut fork = Fork::new(Arc::new(branch), self.grid.clone(), self.series.clone());
        fork.import::<C::Vm>(&head, block)?;
        self.forks.insert(hash, fork);
        Ok(())
    }

    /// Import a new block to the chain.
    ///
    /// returns true if the block is imported.
    pub async fn import(&mut self, block: &Block) -> anyhow::Result<bool> {
        tracing::trace!(
            "importing block#{}@0x{}",
            block.header.slot,
            hex::encode(&block.header.hash()?[..3])
        );
        let head = block.header.head()?;
        if block.header.slot <= self.grandpa.handshake.head.slot {
            tracing::trace!(
                "Discarding block#{}@0x{}... since it's before the finalized block={}",
                head.slot,
                hex::encode(&head.hash[..6]),
                self.grandpa.handshake.head.slot
            );
            return Ok(false);
        }

        // 1. the block is a child of the finalized
        if block.header.parent == self.grandpa.handshake.head.hash {
            tracing::trace!("block is child of the finalized head");
            self.fork(block)?;
            return Ok(true);
        }

        // 2. the block is a child of a fork
        for (_, fork) in self.forks.iter_mut() {
            // 2.1. The block is a child of a fork.
            let head = fork.best()?;
            if head.hash == block.header.parent {
                tracing::trace!("block is a child of a fork");
                fork.import::<C::Vm>(&head, block)?;
                return Ok(true);
            }

            for fhead in fork.chain.iter() {
                // 2.2. The block exists.
                if fhead.hash == head.hash {
                    return Ok(false);
                }

                // 2.3 the block is a fork of a fork
                if fhead.hash == block.header.parent {
                    tracing::trace!("block is on a fork of a fork");
                    let fork = fork.fork::<C::Vm>(fhead, block)?;
                    self.forks.insert(head.hash, fork);
                    return Ok(true);
                }
            }
        }

        // 3. we don't have the ancestors of this block
        tracing::trace!("block is an orphan");
        self.orphan.insert(head.hash, block.clone());
        Ok(false)
    }

    /// Import the genesis block
    pub async fn import_genesis(
        &mut self,
        header: Header,
        state: &HashMap<[u8; 31], Vec<u8>>,
    ) -> anyhow::Result<()> {
        // 1. save the block to the storage
        let head = header.head()?;

        // 2. set the genesis state
        let mut kvs = Vec::new();
        for (key, value) in state {
            kvs.push((*key, value.clone()));
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

        let root = self.state.root()?;
        self.state.commit(Column::State, (kvs, vec![]).into())?;
        self.state.finalize(
            &Block {
                header,
                extrinsic: Default::default(),
            },
            head.hash,
            root,
        )?;

        self.grandpa.handshake.head = head;
        Ok(())
    }
}
