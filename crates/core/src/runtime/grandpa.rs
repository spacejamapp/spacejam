//! Chain related APIs

use crate::{block::Header, OpaqueHash, TimeSlot};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Chain head cache of SpaceJam
pub type Grandpa = Arc<RwLock<Head>>;

/// Chain head cache of SpaceJam
#[derive(Default, Clone)]
pub struct Head {
    /// The head of the chain, e.g. the finalized header.
    pub head: Header,

    /// The leaves of the chain
    pub leaves: Vec<Header>,
}

impl Head {
    /// Check if a child is valid
    pub fn child(&self, _header: Header, _hash: OpaqueHash, _slot: TimeSlot) -> bool {
        true
    }

    /// Update the head of the chain
    pub fn update(&mut self, header: Header) {
        self.head = header;
    }

    /// Add a new leaf to the chain
    pub fn add_leaf(&mut self, header: Header) {
        self.leaves.push(header);
    }
}
