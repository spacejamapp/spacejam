//! Handshake data

use score::{block::Head, OpaqueHash};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Head and unfinalized leaves of the grandpa protocol.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Handshake {
    /// The hash of the head of the chain, e.g. the finalized header.
    ///
    /// This represents the latest block that has been finalized by the GRANDPA protocol.
    pub head: Head,

    /// The leaves of the best finalized head.
    ///
    /// Descendants of the latest finalized block with no known children.
    pub leaves: BTreeSet<Head>,
}

impl Handshake {
    /// Create a new handshake from the given head.
    pub fn new(head: Head) -> Self {
        Self {
            head,
            leaves: Default::default(),
        }
    }

    /// Check if the provided header is acceptable for the peer, it should be
    /// skipped if:
    ///
    /// 1. A descendant of the block is announced instead.
    /// 2. The block is not a descendant of the latest finalized block.
    /// 3. The block, or a descendant of the block, has been announced by the
    ///    other side of the stream.
    ///
    /// 1 and 2 will be checked by our local chain, so this method only checks 3.
    pub fn accept(&self, hash: &OpaqueHash) -> bool {
        self.head.hash != *hash && !self.leaves.iter().any(|head| head.hash == *hash)
    }

    /// Add a leaf to the handshake.
    pub fn add_leaf(&mut self, mut chain: BTreeSet<Head>, leaf: Head) {
        chain.retain(|h| h.slot < leaf.slot);
        self.leaves.insert(leaf);
        self.leaves
            .retain(|head| !chain.iter().any(|h| h.hash == head.hash));
    }
}
