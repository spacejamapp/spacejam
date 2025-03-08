//! Chain related APIs
//!
//! The GRANDPA (GHOST-based Recursive ANcestor Deriving Prefix Agreement) finality
//! protocol implementation for SpaceJam. This module manages chain head finalization
//! and tracks chain state for consensus.

use crate::{
    block::Header,
    runtime::{storage::BlockStorage, Storage},
    OpaqueHash, TimeSlot,
};
use ancestry::Ancestry;
use grid::Grid;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    ops::{Deref, DerefMut},
};

mod ancestry;
mod grid;

/// Chain head cache of SpaceJam
///
/// GRANDPA is responsible for managing the finalized chain head and tracking leaf blocks
/// in the current fork choice tree.
#[derive(Clone, Default)]
pub struct Grandpa {
    /// The hash of the head of the chain, e.g. the finalized header.
    ///
    /// This represents the latest block that has been finalized by the GRANDPA protocol.
    pub head: Head,

    /// The leaves of the best finalized head.
    ///
    /// Descendants of the latest finalized block with no known children.
    pub leaves: HashSet<Head>,

    /// The ancestry of the chain.
    ///
    /// This is a map of block hashes to the set of block hashes that are their ancestors.
    ///
    /// TODO: in production, we should store ancestor blocks in storage, see the header section
    /// from the graypaper for more details.
    pub ancestry: Ancestry,

    /// The grid of the network.
    pub grid: Grid,
}

impl Grandpa {
    /// Create a new grandpa instance.
    pub fn new(storage: &impl Storage) -> anyhow::Result<Self> {
        let head = storage.get_finalized()?;
        let finalized = storage.get_block(&head.hash)?;

        // save finalized block to the ancestry
        //
        // TODO: load all finalized blocks in 24hrs to the ancestry.
        let mut ancestry = Ancestry::default();
        ancestry.save_header(finalized.header)?;

        Ok(Self {
            head,
            leaves: Default::default(),
            ancestry,
            grid: Grid::new(storage)?,
        })
    }

    /// Add a leave to the grandpa.
    ///
    /// If there are ancestors of the leaf in the leaves,
    /// we should remove the ancestors.
    pub fn add_leave(&mut self, head: Head) {
        let ancestors = self
            .ancestors(&head.hash, self.head.hash)
            .iter()
            .filter_map(|h| h.hash().ok())
            .collect::<HashSet<_>>();

        // remove the ancestors from the leaves
        let mut leaves = self.leaves.clone();
        leaves.insert(head);
        leaves.retain(|l| !ancestors.contains(&l.hash));

        // update the leaves
        self.leaves = leaves;
    }

    /// Finalize a head.
    pub fn finalize(&mut self, header: Header) -> anyhow::Result<()> {
        let head = Head {
            hash: header.hash()?,
            slot: header.slot,
        };

        self.head = head.clone();
        self.leaves = self
            .leaves
            .iter()
            .filter(|l| l.hash != head.hash)
            .cloned()
            .collect();

        // TODO: update the grid.

        Ok(())
    }

    /// Select the best head from the leaves.                                                                                                                                                                                                                                                                                                                                                                                           
    pub fn select_best_head(&self) -> Option<Head> {
        let mut votes = BTreeMap::<usize, (Head, Vec<Header>)>::new();

        for leaf in self.leaves.iter() {
            let ancestors = self.ancestors(&leaf.hash, self.head.hash);
            votes.insert(ancestors.len(), (leaf.clone(), ancestors));
        }

        // select the best head from the chains with most valid ancestors, skipping
        // the chains with equivocating ancestors.
        while let Some((_, (head, ancestors))) = votes.pop_last() {
            if ancestors.iter().any(|a| {
                let Some(entry) = self.ancestry.slots.get(&(a.slot, a.parent)) else {
                    return false;
                };

                entry.len() > 1
            }) {
                continue;
            }

            if head.slot > self.head.slot {
                return Some(head);
            }
        }

        None
    }

    /// Create a handshake message for the grandpa protocol.
    pub fn handshake(&self) -> Vec<u8> {
        let mut handshake = vec![];
        handshake.extend_from_slice(self.head.hash.as_ref());
        handshake.extend_from_slice(&self.head.slot.to_le_bytes());
        for head in self.leaves.iter() {
            handshake.extend_from_slice(head.hash.as_ref());
            handshake.extend_from_slice(&head.slot.to_le_bytes());
        }

        handshake
    }

    /// Verify a header with ancestry
    pub async fn verify(&self, header: &Header) -> anyhow::Result<()> {
        let hash = header.hash()?;

        // 1. A descendant of the block is announced instead of the block itself.
        let leaves = self.leaves.iter().filter(|l| l.slot > header.slot);
        for leaf in leaves {
            if !self.is_descendant_of(leaf.hash, hash) {
                anyhow::bail!(
                    "A descendant of the block is announced instead of the block itself."
                );
            }
        }

        // 2. The block is not a descendant of the latest finalized block.
        if !self.is_descendant_of(hash, self.head.hash) {
            anyhow::bail!(
                "block#{}@0x{} is not a descendant of the latest finalized block#{}@0x{}.",
                header.slot,
                hex::encode(hash.as_ref()),
                self.head.slot,
                hex::encode(self.head.hash.as_ref()),
            );
        }

        Ok(())
    }
}

impl Deref for Grandpa {
    type Target = Ancestry;

    fn deref(&self) -> &Self::Target {
        &self.ancestry
    }
}

impl DerefMut for Grandpa {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ancestry
    }
}

/// The head of the chain
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Head {
    /// The hash of the head of the chain.
    pub hash: OpaqueHash,

    /// The slot of this head.
    pub slot: TimeSlot,
}
