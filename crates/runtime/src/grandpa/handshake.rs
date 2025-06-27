//! Handshake data

use score::block::Head;
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
}
