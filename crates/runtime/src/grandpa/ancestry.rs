//! The ancestor map.

use score::{OpaqueHash, block::Header};
use std::collections::{HashMap, HashSet};

/// The ancestor map.
///
/// This API follows the KV store API which is reserved
/// for the future migration to the storage layer.
#[derive(Clone, Default)]
pub struct Ancestry {
    /// The parent of each header.
    pub parent: HashMap<OpaqueHash, OpaqueHash>,

    /// The child of each header.
    pub child: HashMap<OpaqueHash, OpaqueHash>,

    /// The header of each hash.
    header: HashMap<OpaqueHash, Header>,

    /// Mapping from (slot, parent_hash) to block hashes
    ///
    /// This allows detecting true equivocations (same slot AND same parent)
    pub pending: HashMap<OpaqueHash, HashSet<OpaqueHash>>,
}

impl Ancestry {
    /// Save the header to the ancestry.
    pub(crate) fn save_header(&mut self, header: Header) -> anyhow::Result<()> {
        let hash = header.hash()?;
        let parent = header.parent;

        self.parent.insert(hash, parent);
        self.child.insert(parent, hash);
        self.header.insert(hash, header);
        self.pending.entry(parent).or_default().insert(hash);
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
        while let Some(parent) = self.parent.get(&hash) {
            if parent == &ancestor {
                return true;
            }

            hash = *parent;
        }

        false
    }

    /// Get the ancestors of the given head.
    pub fn ancestors(&self, hash: &OpaqueHash, ancestor: OpaqueHash) -> Vec<(OpaqueHash, Header)> {
        let mut ancestors = Vec::new();
        let mut current = *hash;
        while let Some(parent) = self.parent.get(&current) {
            if parent == &ancestor {
                break;
            }

            // if the header is not in the ancestry, break
            let Some(header) = self.header.get(parent) else {
                break;
            };

            ancestors.push((*parent, header.clone()));
            current = *parent;
        }

        ancestors
    }
}
