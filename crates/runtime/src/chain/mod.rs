//! chain of blocks.

use crate::{Config, Grandpa, Storage};
use fork::Fork;
use score::{block::Head, extrinsic::TicketsOrKeys, Block, OpaqueHash, TimeSlot};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

mod fork;
mod importer;

/// A chain of blocks.
pub struct Chain<C: Config> {
    /// The forks of the chain.
    forks: HashMap<OpaqueHash, Fork<C::Storage>>,

    /// The grandpa of the chain.
    grandpa: Grandpa<C::Storage>,

    /// The cached series
    series: BTreeMap<TimeSlot, TicketsOrKeys>,

    /// The storage of the chain.
    state: Arc<C::Storage>,
}

impl<C: Config> Chain<C> {
    /// Create a new chain.
    pub fn new(state: Arc<C::Storage>, grandpa: Grandpa<C::Storage>) -> Self {
        Self {
            forks: HashMap::new(),
            grandpa,
            series: BTreeMap::new(),
            state,
        }
    }

    /// Get the finalized head of the chain.
    pub fn finalized(&self) -> Head {
        self.grandpa.handshake.head.clone()
    }
}
