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
    collections::HashSet,
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
        let curr = storage.current_validators()?;
        let prev = storage.previous_validators()?;
        let next = storage.next_validators()?;

        Ok(Self {
            head,
            leaves: Default::default(),
            ancestry: Default::default(),
            grid: Grid::try_from((
                prev.iter().map(|v| v.ed25519).collect(),
                curr.iter().map(|v| v.ed25519).collect(),
                next.iter().map(|v| v.ed25519).collect(),
            ))?,
        })
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
            if !self.is_descendant_of(&leaf.hash, hash) {
                anyhow::bail!(
                    "A descendant of the block is announced instead of the block itself."
                );
            }
        }

        // 2. The block is not a descendant of the latest finalized block.
        if !self.is_descendant_of(&hash, self.head.hash) {
            anyhow::bail!("The block is not a descendant of the latest finalized block.");
        }

        // 3. if the header is ticket sealed.
        if header.tickets_mark.is_none() {
            anyhow::bail!("The block is not ticket sealed.");
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
