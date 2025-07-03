//! chain of blocks.

use crate::{
    storage::{StateStorage, SyncStorage},
    Config, Grandpa,
};
use score::{extrinsic::TicketsOrKeys, Block, OpaqueHash, TimeSlot};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
pub use {
    fork::Fork,
    grid::Grid,
    importer::Imported,
    lookup::{Direction, Lookup},
};

mod api;
mod author;
mod finalizer;
mod fork;
mod grid;
mod importer;
mod lookup;

/// A chain of blocks.
pub struct Chain<C: Config> {
    /// The forks of the chain.
    pub forks: HashMap<OpaqueHash, Fork<C::Storage>>,

    /// The grandpa of the chain.
    pub grandpa: Grandpa,

    /// The grid of the network.
    grid: Grid,

    /// The orphan blocks.
    orphan: BTreeMap<TimeSlot, BTreeMap<OpaqueHash, Block>>,

    /// The cached series per epoch.
    series: BTreeMap<u32, TicketsOrKeys>,

    /// The storage of the chain.
    state: Arc<C::Storage>,
}

impl<C: Config> Chain<C> {
    /// Create a new chain.
    pub fn new(state: Arc<C::Storage>) -> Self {
        Self {
            forks: HashMap::new(),
            grandpa: Default::default(),
            grid: Grid::default(),
            orphan: BTreeMap::new(),
            series: BTreeMap::new(),
            state,
        }
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

    /// Initialize the chain from the state.
    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        let curr = self.state.current_validators()?;
        let prev = self.state.previous_validators()?;
        let next = self.state.next_validators()?;
        let finalized = self.state.finalized()?;
        tracing::info!("finalized: #{}", finalized.slot);

        self.grid.prev = prev;
        self.grid.curr = curr;
        self.grid.next = next;
        self.grandpa.handshake.head = finalized;
        Ok(())
    }
}
