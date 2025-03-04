//! Chain related APIs
//!
//! The GRANDPA (GHOST-based Recursive ANcestor Deriving Prefix Agreement) finality
//! protocol implementation for SpaceJam. This module manages chain head finalization
//! and tracks chain state for consensus.

use crate::{
    runtime::{storage::BlockStorage, Storage},
    Ed25519Public, OpaqueHash, TimeSlot,
};
use grid::Grid;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

    /// The leaves of the chain.
    ///
    /// These are the tips of all known forks that could potentially become finalized.
    /// Kept in memory for efficiency due to short block time.
    pub leaves: HashMap<Head, HashSet<Ed25519Public>>,

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
        for (head, _) in self.leaves.iter() {
            handshake.extend_from_slice(head.hash.as_ref());
            handshake.extend_from_slice(&head.slot.to_le_bytes());
        }

        handshake
    }
}

/// The head of the chain
#[derive(Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Head {
    /// The hash of the head of the chain.
    pub hash: OpaqueHash,

    /// The slot of this head.
    pub slot: TimeSlot,
}
