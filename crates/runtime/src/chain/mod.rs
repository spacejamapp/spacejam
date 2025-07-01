//! chain of blocks.

use crate::{Config, Grandpa, Handshake};
use score::{
    block::{Head, Header},
    extrinsic::TicketsOrKeys,
    Block, OpaqueHash,
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
pub use {fork::Fork, grid::Grid};

mod fork;
mod grid;
mod importer;

/// A chain of blocks.
pub struct Chain<C: Config> {
    /// The forks of the chain.
    forks: HashMap<OpaqueHash, Fork<C::Storage>>,

    /// The grandpa of the chain.
    pub grandpa: Grandpa<C::Storage>,

    /// The grid of the network.
    grid: Grid,

    /// The orphan blocks.
    orphan: HashMap<OpaqueHash, Block>,

    /// The cached series per epoch.
    series: BTreeMap<u32, TicketsOrKeys>,

    /// The storage of the chain.
    state: Arc<C::Storage>,
}

impl<C: Config> Chain<C> {
    /// Create a new chain.
    pub fn new(state: Arc<C::Storage>) -> Self {
        let grandpa = Grandpa::new(state.clone());
        Self {
            forks: HashMap::new(),
            grandpa,
            grid: Grid::default(),
            orphan: HashMap::new(),
            series: BTreeMap::new(),
            state,
        }
    }

    /// Add a leaf to the handshake.
    ///
    /// Returns `true` if the leaf is already in the chain.
    pub fn add_leaf_to(
        &self,
        head: Head,
        leaf: &Header,
        handshake: &mut Handshake,
    ) -> anyhow::Result<bool> {
        let mut exists = false;
        let mut added = false;
        for fork in self.forks.values() {
            for block in fork.chain.iter() {
                if block.hash == leaf.parent {
                    handshake.add_leaf(fork.chain.clone(), head.clone());
                    added = true;
                }

                if block.hash == head.hash {
                    exists = true;
                }
            }

            if added {
                break;
            }
        }

        Ok(exists)
    }

    /// Select the chain of the given block.
    pub fn contains(&self, block: OpaqueHash) -> bool {
        if self.grandpa.handshake.head.hash == block
            || self
                .grandpa
                .handshake
                .leaves
                .iter()
                .any(|h| h.hash == block)
        {
            return true;
        }

        // check if the block is in the forks
        for fork in self.forks.values() {
            if fork.chain.iter().any(|h| h.hash == block) {
                return true;
            }
        }

        false
    }

    /// Get the finalized head of the chain.
    pub fn finalized(&self) -> Head {
        self.grandpa.handshake.head.clone()
    }
}
