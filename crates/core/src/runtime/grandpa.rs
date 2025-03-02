//! Chain related APIs
//!
//! The GRANDPA (GHOST-based Recursive ANcestor Deriving Prefix Agreement) finality
//! protocol implementation for SpaceJam. This module manages chain head finalization
//! and tracks chain state for consensus.
#![allow(unused)]

use crate::{block::Header, Ed25519Public, OpaqueHash, TimeSlot};
use anyhow::Result;
use std::collections::HashSet;

/// Chain head cache of SpaceJam
///
/// GRANDPA is responsible for managing the finalized chain head and tracking leaf blocks
/// in the current fork choice tree. It implements SpaceJam's finality protocol that allows
/// validators to vote on chains rather than individual blocks. The implementation follows
/// the graypaper specifications for block finalization and best chain selection.
#[derive(Default, Clone)]
pub struct Grandpa {
    /// The head of the chain, e.g. the finalized header.
    ///
    /// This represents the latest block that has been finalized by the GRANDPA protocol.
    pub head: Header,

    /// The leaves of the chain.
    ///
    /// These are the tips of all known forks that could potentially become finalized.
    /// Kept in memory for efficiency due to short block time.
    pub leaves: Vec<Head>,

    /// The set of blocks that have received votes but are not yet finalized
    pending_blocks: Vec<(OpaqueHash, Vec<Ed25519Public>)>,

    /// Set of validators
    validators: HashSet<Ed25519Public>,
}

impl Grandpa {
    /// Handle the handshake message
    ///
    /// 1. if the newly connected node has longer finalized chain, we'll verify the proof and
    ///     sync to their state.
    ///     1.1. (inner logic for verifying the finalized blocks)
    /// 2. if the new newly connected node is at the same slot as us, we'll add the leaves to our
    ///     pending list.
    /// 3. if the newly connected node is behind, we'll do nothing.
    ///
    /// Note that the handshake message could only be called for once on each connection.
    pub fn handshake(_hash: OpaqueHash, _slot: TimeSlot, _leaves: Vec<OpaqueHash>) -> Result<()> {
        Ok(())
    }

    /// Handle the block announcement message
    ///
    /// 1. if the block is already in the pending list, we'll do nothing.
    /// 2. if the block is not in the pending list, we'll add it to the pending list.
    /// 3. if the block is in the pending list, we'll update the pending list.
    ///
    /// Note that the block announcement message will be called multiple times on each connection.
    pub fn block_announcement(
        _hash: OpaqueHash,
        _slot: TimeSlot,
        _leaves: Vec<OpaqueHash>,
    ) -> Result<()> {
        Ok(())
    }

    /// Finalize the best chain.
    ///
    /// TODO: maybe this should be called in a timer?
    pub fn finalize(&self) -> Result<()> {
        Ok(())
    }
}

/// Vote for a candidate longest chain
///
/// Represents a chain that meets the criteria for being a potential best head
/// according to the graypaper.
#[derive(Clone, Debug)]
pub struct Head {
    /// The block hash
    pub hash: OpaqueHash,

    /// The slot of the block
    pub slot: TimeSlot,

    /// The validators who have voted for this chain
    validators: Vec<Ed25519Public>,

    /// The number of ancestor blocks that used a seal-key ticket
    /// This is the value 'm' in the graypaper formula that we aim to maximize
    seal_key_ancestors: usize,

    /// The header of this potential best block
    header: Header,
}

#[cfg(test)]
mod tests {}
