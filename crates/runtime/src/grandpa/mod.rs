//! Chain related APIs
//!
//! The GRANDPA (GHOST-based Recursive ANcestor Deriving Prefix Agreement) finality
//! protocol implementation for SpaceJam. This module manages chain head finalization
//! and tracks chain state for consensus.

use ancestry::Ancestry;
pub use handshake::Handshake;
use score::{
    OpaqueHash,
    block::{Head, Header},
    safrole::ValidatorsData,
};
use std::{
    collections::{BTreeMap, HashSet},
    ops::{Deref, DerefMut},
};
use {grid::Grid, lookup::Lookup};

mod ancestry;
mod grid;
mod handshake;
mod lookup;

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
    /// Lookup the ancestors of the given hash.
    pub fn lookup(
        &self,
        hash: OpaqueHash,
        direction: u8,
        maximum: u32,
    ) -> impl Iterator<Item = (OpaqueHash, Header)> + '_ {
        Lookup::new(&self.ancestry, hash, direction, maximum)
    }

    /// Add a leave to the grandpa.
    ///
    /// If there are ancestors of the leaf in the leaves,
    /// we should remove the ancestors.
    pub fn add_leaf(&mut self, header: Header) -> anyhow::Result<()> {
        // We're copying the handshake here because it takes less memory than
        // cloning the whole grandpa.
        let mut handshake = self.handshake.clone();
        self.add_leaf_to(header.clone().try_into()?, &mut handshake)?;
        self.handshake = handshake;
        self.ancestry.save_header(header)?;
        Ok(())
    }

    /// Merge the leaves with the given header.
    pub fn add_leaf_to(&self, head: Head, handshake: &mut Handshake) -> anyhow::Result<()> {
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
            self.grid.prev = self.grid.curr;
            self.grid.curr = self.grid.next;
            self.grid.next = mark;
        }

        Ok(())
    }

    /// Select the best head from the leaves.
    ///
    /// 1. must has the finalized block as an ancestor.
    /// 2. contains no unfinalized blocks where we see an equivocation.
    /// 3. is considered audited
    ///
    /// TODO:
    ///
    /// - count votes via sealed blocks
    pub fn select_best_head(&self) -> Option<(Head, Vec<(OpaqueHash, Header)>)> {
        let mut votes = BTreeMap::new();
        for leaf in self.handshake.leaves.iter() {
            let ancestors = self.ancestors(&leaf.hash, self.handshake.head.hash);
            let valid_ancestors = ancestors.iter().collect::<Vec<_>>();
            votes.insert(valid_ancestors.len(), (leaf.clone(), ancestors));
        }

        // select the best head from the chains with most valid ancestors, skipping
        // the chains with equivocating ancestors.
        while let Some((_, (head, ancestors))) = votes.pop_last() {
            if ancestors.iter().any(|(_, a)| {
                let Some(entry) = self.ancestry.pending.get(&a.parent) else {
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

    /// If a header is acceptable for a remote peer, returns error if:
    ///
    /// 1. The block has been announced by the remote
    /// 2. A descendant of the block has been announced by the remote.
    pub async fn accept_remote(
        &self,
        header: &Header,
        handshake: &Handshake,
    ) -> anyhow::Result<Head> {
        let head = Head::try_from(header.clone())?;
        let shash = hex::encode(&head.hash.as_ref()[..3]);

        // 1. if the block has been announced by the remote
        if handshake.leaves.contains(&head) {
            anyhow::bail!(
                "block#{}@0x{} has been announced by the remote",
                head.slot,
                shash
            );
        }

        // 2. skip if a descendant of the block has been announced by the remote.
        //
        // # Note
        //
        // we can only check with our local state here.
        let leaves = self.handshake.leaves.iter().filter(|l| l.slot > head.slot);
        for leaf in leaves {
            if self.is_descendant_of(leaf.hash, head.hash) {
                anyhow::bail!(
                    "A descendant of the block#{}: 0x{} has been announced by the remote",
                    leaf.slot,
                    hex::encode(&leaf.hash.as_ref()[..3]),
                );
            }
        }

        Ok(head)
    }

    /// If a header is acceptable for local, returns error if:
    ///
    /// 1. A descendant of the block is announced instead
    /// 2. The block is not a descendant of the latest finalized block.
    pub async fn accept_local(&self, head: &Header) -> anyhow::Result<()> {
        let hash = head.hash()?;

        // 1. A descendant of the block is announced instead.
        let leaves = self.handshake.leaves.iter().filter(|l| l.slot > head.slot);
        for leaf in leaves {
            if self.is_descendant_of(leaf.hash, hash) {
                anyhow::bail!(
                    "A descendant of the block#{}@0x{} is announced instead.",
                    leaf.slot,
                    hex::encode(&leaf.hash.as_ref()[..3])
                );
            }
        }

        // 2. The block is not a descendant of the latest finalized block.
        if !self.is_descendant_of(hash, self.handshake.head.hash) {
            anyhow::bail!(
                "block#{}@0x{} is not a descendant of the latest finalized block#{}@0x{}.",
                head.slot,
                hex::encode(&hash.as_ref()[..3]),
                self.handshake.head.slot,
                hex::encode(&self.handshake.head.hash.as_ref()[..3]),
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
