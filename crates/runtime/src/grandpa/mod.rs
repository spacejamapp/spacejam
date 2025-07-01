//! Chain related APIs
//!
//! The GRANDPA (GHOST-based Recursive ANcestor Deriving Prefix Agreement) finality
//! protocol implementation for SpaceJam. This module manages chain head finalization
//! and tracks chain state for consensus.

use grid::Grid;
pub use {ancestry::Ancestry, handshake::Handshake};

mod ancestry;
mod grid;
mod handshake;

/// Chain head cache of SpaceJam
///
/// GRANDPA is responsible for managing the finalized chain head and tracking leaf blocks
/// in the current fork choice tree.
#[derive(Default)]
pub struct Grandpa {
    /// The handshake data of the grandpa protocol.
    pub handshake: Handshake,

    /// The grid of the network.
    pub grid: Grid,
}

impl Clone for Grandpa {
    fn clone(&self) -> Self {
        Self {
            handshake: self.handshake.clone(),
            grid: self.grid.clone(),
        }
    }
}
