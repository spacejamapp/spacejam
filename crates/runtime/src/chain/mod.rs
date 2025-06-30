//! chain of blocks.

use crate::{Grandpa, Storage};
use fork::Fork;
use score::{block::Head, extrinsic::TicketsOrKeys, Block, OpaqueHash, TimeSlot};
use std::collections::BTreeMap;

mod fork;
mod importer;

/// A chain of blocks.
pub struct Chain<S: Storage> {
    /// The forks of the chain.
    forks: Vec<Fork>,

    /// The grandpa of the chain.
    grandpa: Grandpa<S>,

    /// The queued blocks.
    queue: BTreeMap<TimeSlot, BTreeMap<OpaqueHash, Block>>,

    /// The cached series
    series: BTreeMap<TimeSlot, TicketsOrKeys>,

    /// The storage of the chain.
    state: S,
}

impl<S: Storage> Chain<S> {
    /// Create a new chain.
    pub fn new(state: S, grandpa: Grandpa<S>) -> Self {
        Self {
            forks: vec![],
            grandpa,
            queue: BTreeMap::new(),
            series: BTreeMap::new(),
            state,
        }
    }

    /// Get the finalized head of the chain.
    pub fn finalized(&self) -> Head {
        self.grandpa.handshake.head.clone()
    }
}
