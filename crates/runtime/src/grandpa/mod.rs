//! Chain related APIs
//!
//! The GRANDPA (GHOST-based Recursive ANcestor Deriving Prefix Agreement) finality
//! protocol implementation for SpaceJam. This module manages chain head finalization
//! and tracks chain state for consensus.

use crate::{storage::SyncStorage, Storage};
use score::{
    block::{Head, Header},
    safrole::ValidatorsData,
    OpaqueHash,
};
use std::{collections::BTreeSet, sync::Arc};
pub use {ancestry::Ancestry, handshake::Handshake};
use {grid::Grid, lookup::Lookup};

mod ancestry;
mod grid;
mod handshake;
mod lookup;

/// Chain head cache of SpaceJam
///
/// GRANDPA is responsible for managing the finalized chain head and tracking leaf blocks
/// in the current fork choice tree.
pub struct Grandpa<T: Storage> {
    /// The handshake data of the grandpa protocol.
    pub handshake: Handshake,

    /// The ancestry of the chain.
    pub ancestry: Arc<T>,

    /// The grid of the network.
    pub grid: Grid,
}

impl<T: Storage> Grandpa<T> {
    /// Create a new grandpa.
    pub fn new(ancestry: Arc<T>) -> Self {
        Self {
            handshake: Default::default(),
            ancestry,
            grid: Default::default(),
        }
    }

    /// Lookup the ancestors of the given hash.
    pub fn lookup(
        &self,
        hash: OpaqueHash,
        direction: u8,
        maximum: u32,
    ) -> impl Iterator<Item = (OpaqueHash, Header)> + '_ {
        Lookup::new(self.ancestry.clone(), hash, direction, maximum)
    }

    /// Add a leave to the grandpa.
    ///
    /// If there are ancestors of the leaf in the leaves,
    /// we should remove the ancestors.
    pub fn add_leaf(&mut self, header: Header) -> anyhow::Result<()> {
        // Store the header first so parent information is available for ancestor traversal
        self.ancestry.set_header(&header)?;

        // We're copying the handshake here because it takes less memory than
        // cloning the whole grandpa.
        let mut handshake = self.handshake.clone();
        self.add_leaf_to(header.head()?, &mut handshake)?;
        self.handshake = handshake;
        Ok(())
    }

    /// Merge the leaves with the given header.
    pub fn add_leaf_to(&self, head: Head, handshake: &mut Handshake) -> anyhow::Result<()> {
        let ancestors = self.ancestry.ancestors(&head.hash, &handshake.head.hash);
        handshake.leaves.insert(head);
        handshake.leaves.retain(|l| !ancestors.contains(&l.hash));
        Ok(())
    }

    /// Finalize a head.
    pub fn finalize(
        &mut self,
        header: Header,
        next_validators: Option<ValidatorsData>,
    ) -> anyhow::Result<()> {
        let head = Head {
            hash: header.hash()?,
            slot: header.slot,
        };

        self.handshake.head = head.clone();
        self.handshake.leaves = self
            .handshake
            .leaves
            .iter()
            .filter(|l| l.slot > head.slot)
            .cloned()
            .collect::<BTreeSet<_>>();

        // save to the ancestry
        self.ancestry.set_header(&header)?;

        // if new epoch start
        if let Some(mark) = next_validators {
            self.grid.prev = self.grid.curr;
            self.grid.curr = self.grid.next;
            self.grid.next = mark;
        }

        Ok(())
    }
}

impl<T: Storage> Clone for Grandpa<T> {
    fn clone(&self) -> Self {
        Self {
            handshake: self.handshake.clone(),
            ancestry: self.ancestry.clone(),
            grid: self.grid.clone(),
        }
    }
}
