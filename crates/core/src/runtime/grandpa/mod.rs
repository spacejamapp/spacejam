//! Chain related APIs
//!
//! The GRANDPA (GHOST-based Recursive ANcestor Deriving Prefix Agreement) finality
//! protocol implementation for SpaceJam. This module manages chain head finalization
//! and tracks chain state for consensus.

use crate::{block::Header, safrole::ValidatorsData, OpaqueHash, TimeSlot};
use ancestry::Ancestry;
use grid::Grid;
pub use handshake::Handshake;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    ops::{Deref, DerefMut},
};

mod ancestry;
mod grid;
mod handshake;

/// Chain head cache of SpaceJam
///
/// GRANDPA is responsible for managing the finalized chain head and tracking leaf blocks
/// in the current fork choice tree.
#[derive(Clone, Default)]
pub struct Grandpa {
    /// The handshake data of the grandpa protocol.
    pub handshake: Handshake,

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
    /// Add a leave to the grandpa.
    ///
    /// If there are ancestors of the leaf in the leaves,
    /// we should remove the ancestors.
    pub fn add_leaf(&mut self, header: Header) -> anyhow::Result<()> {
        // We're copying the handshake here because it takes less memory than
        // cloning the whole grandpa.
        let mut handshake = self.handshake.clone();
        self.add_leaf_to(&header, &mut handshake)?;
        self.handshake = handshake;
        self.ancestry.save_header(header)?;
        Ok(())
    }

    /// Merge the leaves with the given header.
    pub fn add_leaf_to(&self, header: &Header, handshake: &mut Handshake) -> anyhow::Result<()> {
        let head = Head {
            hash: header.hash()?,
            slot: header.slot,
        };

        let ancestors = self
            .ancestors(&head.hash, handshake.head.hash)
            .iter()
            .map(|(h, _)| *h)
            .collect::<HashSet<_>>();

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
            .filter(|l| l.hash != head.hash)
            .cloned()
            .collect::<HashSet<_>>();

        // save to the ancestry
        self.ancestry.save_header(header.clone())?;

        // if new epoch start
        if let Some(mark) = next_validators {
            self.grid.prev = self.grid.curr.clone();
            self.grid.curr = self.grid.next.clone();
            self.grid.next = mark;
        }

        Ok(())
    }

    /// Select the best head from the leaves.
    ///
    /// 1. must has the finalized block as an ancestor.
    /// 2. contains no unfinalized blocks where we see an equivocation.
    /// 3. is considered audit
    /// 4. the best head must be ticket sealed
    pub fn select_best_head(&self) -> Option<(Head, Vec<(OpaqueHash, Header)>)> {
        let mut votes = BTreeMap::new();
        for leaf in self.handshake.leaves.iter() {
            let ancestors = self.ancestors(&leaf.hash, self.handshake.head.hash);
            let valid_ancestors = ancestors
                .iter()
                .filter_map(|(_, h)| {
                    if h.tickets_mark.is_some() {
                        Some(h)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            votes.insert(valid_ancestors.len(), (leaf.clone(), ancestors));
        }

        // select the best head from the chains with most valid ancestors, skipping
        // the chains with equivocating ancestors.
        while let Some((_, (head, ancestors))) = votes.pop_last() {
            if ancestors.iter().any(|(_, a)| {
                let Some(entry) = self.ancestry.slots.get(&(a.slot, a.parent)) else {
                    return false;
                };

                entry.len() > 1
            }) {
                continue;
            }

            if head.slot > self.handshake.head.slot {
                let Some(_header) = self.ancestry.header(&head.hash) else {
                    continue;
                };

                // TODO: check if the header is ticket sealed
                return Some((head, ancestors));
            }
        }

        None
    }

    /// Verify a header with ancestry
    pub async fn verify(&self, header: &Header) -> anyhow::Result<()> {
        let hash = header.hash()?;

        // 1. A descendant of the block is announced instead of the block itself.
        //
        // Compare the header with the leaves.
        let leaves = self
            .handshake
            .leaves
            .iter()
            .filter(|l| l.slot > header.slot);
        for leaf in leaves {
            if leaf.hash == hash {
                return Ok(());
            }

            if !self.is_descendant_of(leaf.hash, hash) {
                anyhow::bail!(
                    "A descendant of the block is announced instead of the block itself."
                );
            }
        }

        // 2. if the header is directly the child of the latest finalized block,
        // we should check if the header is ticket sealed.
        //
        // we need to check this directly because we may not have the info of a
        // newly incoming header.
        if header.parent == self.handshake.head.hash {
            return Ok(());
        }

        // 3. The block is not a descendant of the latest finalized block.
        //
        // We are using the parent of the header because a new header will not be
        // registered in the ancestry yet.
        if !self.is_descendant_of(header.parent, self.handshake.head.hash) {
            anyhow::bail!(
                "block#{}@0x{} is not a descendant of the latest finalized block#{}@0x{}, parent: 0x{}.",
                header.slot,
                hex::encode(&hash.as_ref()[..3]),
                self.handshake.head.slot,
                hex::encode(&self.handshake.head.hash.as_ref()[..3]),
                hex::encode(&header.parent.as_ref()[..3]),
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
