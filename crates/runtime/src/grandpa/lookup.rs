//! Block lookup APIs

use crate::grandpa::Ancestry;
use score::{block::Header, OpaqueHash};

/// The lookup data of the grandpa.
pub struct Lookup<'a> {
    /// The ancestry of the chain.
    ancestry: &'a Ancestry,

    /// The current hash.
    pub current: OpaqueHash,

    /// The direction of the lookup.
    pub direction: u8,

    /// The maximum number of blocks to lookup.
    maximum: u32,

    /// The number of blocks already looked up.
    count: u32,
}

impl<'a> Lookup<'a> {
    /// Create a new lookup.
    pub fn new(ancestry: &'a Ancestry, from: OpaqueHash, direction: u8, maximum: u32) -> Self {
        Self {
            ancestry,
            current: from,
            direction,
            count: 0,
            maximum,
        }
    }
}

impl Iterator for Lookup<'_> {
    type Item = (OpaqueHash, Header);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.maximum {
            return None;
        }

        // get the next hash
        let hash = match self.direction {
            0 => self.ancestry.child.get(&self.current).cloned(),
            1 => {
                if self.count == 0 {
                    Some(self.current)
                } else {
                    self.ancestry.child.get(&self.current).cloned()
                }
            }
            _ => None,
        }?;

        // get the header
        let header = self.ancestry.header(&hash).cloned()?;
        self.current = hash;
        self.count += 1;
        Some((hash, header))
    }
}
