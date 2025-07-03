//! Importer interface for the chain.

use crate::{
    chain::fork::Fork,
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
    pub fn best_chain(&self) -> Result<Fork<C::Storage>> {
        let mut count = 0;
        let mut best = None;
        for (hash, fork) in self.forks.iter() {
            let flen = fork.len();
            if flen > count {
                count = flen;
                best = Some(hash);
            }
        }

        best.and_then(|hash| self.forks.get(hash).cloned())
            .ok_or_else(|| anyhow::anyhow!("could not find the best chain"))
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
    pub fn import(&mut self, block: &Block) -> anyhow::Result<Imported> {
        let head = block.header.head()?;
        if block.header.slot <= self.grandpa.handshake.head.slot
            || self
                .orphan
                .get(&head.slot)
                .and_then(|o| o.get(&head.hash))
                .is_some()
        {
            tracing::trace!(
                "Discarding block#{}@0x{}...",
                head.slot,
                hex::encode(&head.hash[..6]),
            );
            return Ok(Imported::Discarded);
        }

        tracing::trace!(
            "importing block#{}@0x{}, parent=0x{}",
            head.slot,
            hex::encode(&head.hash[..3]),
            hex::encode(&block.header.parent[..3])
        );

        // 1 the block is already imported
        if self.forks.iter().any(|(fhead, _fork)| fhead == &head.hash) {
            tracing::trace!("block is already imported");
            return Ok(Imported::Skipped);
        }

        // 2. the block is a child of the finalized
        if block.header.parent == self.grandpa.handshake.head.hash {
            tracing::trace!("block is child of the finalized head");
            self.fork(block)?;
            return Ok(Imported::Finalized);
        }

        // 3. the block is a child of a fork
        for (_, fork) in self.forks.iter_mut() {
            // 3.1. The block is a child of a fork.
            let best = fork.best()?;
            if best.hash == block.header.parent {
                tracing::trace!("block is a child of a fork");
                fork.import::<C::Vm>(&best, block)?;
                return Ok(Imported::Fork);
            }

            // 3.2 block is already imported
            if fork.chain.iter().any(|h| h.hash == head.hash) {
                tracing::trace!("block is already imported");
                return Ok(Imported::Skipped);
            }

            // 3.3 the block is a fork of a fork
            for fhead in fork.chain.iter() {
                if fhead.hash == block.header.parent {
                    tracing::trace!("block is on a fork of a fork");
                    let fork = fork.fork::<C::Vm>(fhead, block)?;
                    self.forks.insert(head.hash, fork);
                    return Ok(Imported::ForkOfFork);
                }
            }
        }

        // 4. we don't have the ancestors of this block
        tracing::trace!("block is an orphan");
        self.orphan
            .entry(head.slot)
            .or_default()
            .insert(head.hash, block.clone());
        Ok(Imported::Orphan)
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

    /// Resolve the orphan blocks
    pub fn resolve_orphan(&mut self) -> anyhow::Result<()> {
        let mut imported = vec![];
        let orphan = self.orphan.clone();
        for (slot, orphans) in orphan.iter() {
            for (hash, block) in orphans.iter() {
                if self.import(block)?.imported() {
                    imported.push((slot, hash));
                }
            }
        }

        for (slot, hash) in imported {
            self.orphan.entry(*slot).or_default().remove(hash);
        }

        self.orphan.retain(|_, orphans| !orphans.is_empty());
        Ok(())
    }
}

/// The imported status
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Imported {
    /// The block is discarded
    Discarded,
    /// The block is skipped
    Skipped,
    /// A child of finalized block
    Finalized,
    /// A child of a fork
    Fork,
    /// A child of a fork of a fork
    ForkOfFork,
    /// An orphan block
    Orphan,
}

impl Imported {
    /// Returns true if the block is imported
    pub fn imported(&self) -> bool {
        matches!(
            self,
            Imported::Finalized | Imported::Fork | Imported::ForkOfFork
        )
    }

    /// Returns true if the block is orphan
    pub fn is_orphan(&self) -> bool {
        *self == Imported::Orphan
    }
}
