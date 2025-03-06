//! The ancestor map.

use crate::{block::Header, OpaqueHash};
use std::collections::HashMap;

/// The ancestor map.
///
/// This API follows the KV store API which is reserved
/// for the future migration to the storage layer.
#[derive(Clone, Default)]
pub struct Ancestry {
    /// The parent of each header.
    parent: HashMap<OpaqueHash, OpaqueHash>,

    /// The header of each hash.
    header: HashMap<OpaqueHash, Header>,
}

impl Ancestry {
    /// Save the header to the ancestry.
    pub fn save_header(&mut self, header: Header) -> anyhow::Result<()> {
        let hash = header.hash()?;
        self.parent.insert(hash, header.parent);
        self.header.insert(hash, header);
        Ok(())
    }

    /// Check if the given hash is a descendant of the current hash.
    ///
    /// TODO: set the limit of 24 hrs (MAX_AGE_LOOKUP_ANCHOR)
    pub fn is_descendant_of(&self, hash: &OpaqueHash, mut ancestor: OpaqueHash) -> bool {
        while let Some(parent) = self.parent.get(&ancestor) {
            if parent == hash {
                return true;
            }

            ancestor = *parent;
        }

        false
    }

    /// Get the ticket sealed ancestors count of the given head.
    ///
    /// Which is also the votes of this head.
    pub fn ancestors(&self, hash: &OpaqueHash, finalized: OpaqueHash) -> Vec<Header> {
        let mut ancestors: Vec<Header> = Vec::new();
        let mut ancestor = *hash;
        while let Some(parent) = self.parent.get(&ancestor) {
            if parent == &finalized {
                break;
            }

            // if the header is not in the ancestry, break
            let Some(header) = self.header.get(&ancestor) else {
                break;
            };

            // if the header is not a ticket sealed header, break
            if header.tickets_mark.is_none() {
                continue;
            }

            ancestors.push(header.clone());
            ancestor = *parent;
        }

        ancestors
    }
}
