//! Importer interface for the chain.

use crate::{
    chain::fork::Fork,
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
    pub fn finalize(&mut self) -> Result<()> {
        let best = self.best()?;
        let fork = self
            .forks
            .get_mut(&best.hash)
            .ok_or_else(|| anyhow::anyhow!("could not find the best chain"))?;

        // nothing to finalize if the best fork chain is less than 5 blocks.
        if fork.len() < 5 {
            return Ok(());
        }

        // now we need to truncate all forks.

        Ok(())
    }

    /// Create a new fork at the latest finalized block.
    pub fn fork(&mut self, block: &Block) -> Result<()> {
        let hash = block.header.hash()?;
        let branch = Branch::checkout(self.state.clone());
        let mut fork = Fork::new(branch, self.series.clone());
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
        //
        // NOTE: This should have been checked in the grandpa before this
        // function is called. however we still check it here to be safe.
        anyhow::bail!(
            "block#{}@0x{} is not a child of the finalized block or a fork",
            head.slot,
            hex::encode(&head.hash[..3])
        );
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
                    self.grandpa.grid.prev = codec::decode(value)?;
                }
                key::CURRENT_VALIDATORS => {
                    self.grandpa.grid.curr = codec::decode(value)?;
                }
                key::NEXT_VALIDATORS => {
                    self.grandpa.grid.next = codec::decode(value)?;
                }
                _ => {}
            }
        }

        self.state.commit((kvs, vec![]).into())?;
        self.grandpa.handshake.head = head;
        Ok(())
    }
}
