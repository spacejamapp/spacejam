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
        self.add_leaf_to(header.clone().try_into()?, &mut handshake)?;
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

    /// Select the best head from the leaves.
    ///
    /// 1. must has the finalized block as an ancestor.
    /// 2. contains no unfinalized blocks where we see an equivocation.
    /// 3. is considered audited
    ///
    /// NOTE: we are now using a dummy finalizer which finalizes when
    /// the best head has 5 descendants.
    ///
    /// TODO:
    ///
    /// - count votes via sealed blocks
    pub fn select_best_head(&self) -> Option<Ancestry> {
        let finalized = self.handshake.head.clone();
        let mut selected = if let Ok(best) = self.ancestry.best() {
            let ancestors = self.ancestry.ancestors(&best.hash, &finalized.hash);

            Some((best, ancestors))
        } else {
            None
        };

        for leaf in self.handshake.leaves.iter().rev() {
            let ancestors = self.ancestry.ancestors(&leaf.hash, &finalized.hash);
            let Some((_, chain)) = &selected else {
                selected = Some((leaf.clone(), ancestors));
                continue;
            };

            if ancestors.len() <= chain.len() {
                continue;
            }

            selected = Some((leaf.clone(), ancestors));
        }

        let (best, ancestors) = selected?;
        Some(Ancestry {
            best,
            ancestors,
            finalized,
        })
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
            if self.ancestry.is_descendant_of(&leaf.hash, &head.hash) {
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
            if self.ancestry.is_descendant_of(&leaf.hash, &hash) {
                anyhow::bail!(
                    "A descendant of the block#{}@0x{} is announced instead.",
                    leaf.slot,
                    hex::encode(&leaf.hash.as_ref()[..3])
                );
            }
        }

        // 2. The block is not a descendant of the latest finalized block.
        if !self
            .ancestry
            .is_descendant_of(&hash, &self.handshake.head.hash)
        {
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

impl<T: Storage> Clone for Grandpa<T> {
    fn clone(&self) -> Self {
        Self {
            handshake: self.handshake.clone(),
            ancestry: self.ancestry.clone(),
            grid: self.grid.clone(),
        }
    }
}
