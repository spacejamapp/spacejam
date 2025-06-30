//! Importer interface for the chain.

use crate::{
    chain::fork::Fork,
    storage::{Branch, KVStorage, SyncStorage},
    Chain, Config,
};
use anyhow::Result;
use score::{block::Header, state::key, Block, OpaqueHash};
use std::collections::HashMap;

impl<C: Config> Chain<C> {
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

    /// Import a new block to the chain.
    ///
    /// 1. find the parent of the block
    /// 2. check if the block is a descendant of any forks
    /// 3. import the block to the forks
    pub async fn import(&mut self, block: &Block) -> anyhow::Result<()> {
        let head = block.header.head()?;
        let Some(fork) = self.parent(block.header.parent) else {
            self.queue
                .entry(head.slot)
                .or_default()
                .insert(head.hash, block.clone());
            return Ok(());
        };

        Ok(())
    }

    /// Get the parent of the block.
    pub fn parent(&mut self, hash: OpaqueHash) -> Option<&mut Fork<C::Storage>> {
        for fork in self.forks.values_mut() {
            let len = fork.chain.len();
            for head in fork.chain.iter() {
                if head.hash == hash {
                    return Some(fork);
                }
            }
        }
        None
    }

    /// Create a new fork at the latest finalized block.
    pub fn fork(&mut self, block: &Block) -> Result<()> {
        let branch = Branch::checkout(self.state.clone());
        let mut fork = Fork::new(branch, self.series.clone());
        let hash = block.header.hash()?;
        fork.import::<C::Vm>(block)?;
        self.forks.insert(hash, fork);
        Ok(())
    }
}
