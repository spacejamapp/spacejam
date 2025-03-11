//! The ancestor map.

use crate::{block::Header, OpaqueHash, TimeSlot};
use std::collections::{HashMap, HashSet};

/// The ancestor map.
///
/// This API follows the KV store API which is reserved
/// for the future migration to the storage layer.
#[derive(Clone, Default)]
pub struct Ancestry {
    /// The parent of each header.
    pub parent: HashMap<OpaqueHash, OpaqueHash>,

    /// The header of each hash.
    header: HashMap<OpaqueHash, Header>,

    /// Mapping from (slot, parent_hash) to block hashes
    ///
    /// This allows detecting true equivocations (same slot AND same parent)
    pub slots: HashMap<(TimeSlot, OpaqueHash), HashSet<OpaqueHash>>,
}

impl Ancestry {
    /// Save the header to the ancestry.
    pub(crate) fn save_header(&mut self, header: Header) -> anyhow::Result<()> {
        let hash = header.hash()?;
        let parent = header.parent;
        let slot = header.slot;

        self.parent.insert(hash, parent);
        self.header.insert(hash, header);
        self.slots.entry((slot, parent)).or_default().insert(hash);
        Ok(())
    }

    /// Get the header of the given hash.
    pub fn header(&self, hash: &OpaqueHash) -> Option<&Header> {
        self.header.get(hash)
    }

    /// Check if the given hash is a descendant of the current hash.
    ///
    /// TODO: set the limit of 24 hrs (MAX_AGE_LOOKUP_ANCHOR)
    pub fn is_descendant_of(&self, mut hash: OpaqueHash, ancestor: OpaqueHash) -> bool {
        if hash == ancestor {
            return true;
        }

        while let Some(parent) = self.parent.get(&hash) {
            if parent == &ancestor {
                return true;
            }

            hash = *parent;
        }

        false
    }

    /// Get the ticket sealed ancestors count of the given head.
    ///
    /// Which is also the votes of this head.
    pub fn ancestors(&self, hash: &OpaqueHash, finalized: OpaqueHash) -> Vec<(OpaqueHash, Header)> {
        let mut ancestors = Vec::new();
        let mut ancestor = *hash;
        while let Some(parent) = self.parent.get(&ancestor) {
            if parent == &finalized {
                break;
            }

            // if the header is not in the ancestry, break
            let Some(header) = self.header.get(parent) else {
                break;
            };

            ancestors.push((*parent, header.clone()));
            ancestor = *parent;
        }

        ancestors
    }
}
