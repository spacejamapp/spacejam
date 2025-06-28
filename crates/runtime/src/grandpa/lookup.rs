//! Block lookup APIs

use crate::storage::SyncStorage;
use score::{block::Header, OpaqueHash};
use std::sync::Arc;

/// The lookup data of the grandpa.
pub struct Lookup<T: SyncStorage> {
    /// The ancestry of the chain.
    ancestry: Arc<T>,

    /// The current hash.
    pub current: OpaqueHash,

    /// The direction of the lookup.
    pub direction: u8,

    /// The maximum number of blocks to lookup.
    maximum: u32,

    /// The number of blocks already looked up.
    count: u32,
}

impl<T: SyncStorage> Lookup<T> {
    /// Create a new lookup.
    pub fn new(ancestry: Arc<T>, from: OpaqueHash, direction: u8, maximum: u32) -> Self {
        Self {
            ancestry,
            current: from,
            direction,
            count: 0,
            maximum,
        }
    }
}

impl<T: SyncStorage> Iterator for Lookup<T> {
    type Item = (OpaqueHash, Header);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.maximum {
            return None;
        }

        // get the next hash
        let hash = match self.direction {
            0 => self.ancestry.descendant(&self.current).ok(),
            1 => {
                if self.count == 0 {
                    Some(self.current)
                } else {
                    self.ancestry.parent(&self.current).ok().flatten()
                }
            }
            _ => None,
        }?;

        // get the header
        let header = self.ancestry.header(&hash).ok()?;
        self.current = hash;
        self.count += 1;
        Some((hash, header))
    }
}
