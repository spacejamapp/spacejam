//! Block history

use crate::{
    block::{BlockInfo, BlockInfoJson},
    service::ReportedWorkPackage,
    OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a peak in the Merkle Mountain Range (MMR).
pub type MmrPeak = Option<OpaqueHash>;

/// Block history
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct History {
    /// The history
    #[json(Vec<BlockInfoJson>)]
    pub history: Vec<BlockInfo>,

    /// The Merkle Mountain Range
    #[json(nested)]
    pub mmr: Mmr,
}

/// Represents the Merkle Mountain Range (MMR).
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct Mmr {
    #[json(Vec<Option<String>>)]
    pub peaks: Vec<MmrPeak>,
}

#[cfg(feature = "crypto")]
mod crypto_impl {
    use super::*;
    use crate::{block::BlockInfo, MAX_BLOCKS_HISTORY};
    use crypto::merkle::mmr;

    impl History {
        /// Import a new block into the history
        pub fn import(
            &mut self,
            header_hash: OpaqueHash,
            parent_state_root: OpaqueHash,
            accumulate_root: OpaqueHash,
            reported: Vec<ReportedWorkPackage>,
        ) {
            self.mmr.append(accumulate_root);
            let Some(last) = self.history.last_mut() else {
                let new_block = BlockInfo {
                    header_hash,
                    beefy_root: accumulate_root,
                    state_root: OpaqueHash::default(),
                    reported,
                };
                self.history.push(new_block.clone());
                return;
            };

            // Update the state root of the parent block if it exists
            last.state_root = parent_state_root;
            let new_block = BlockInfo {
                header_hash,
                beefy_root: self.mmr.root().unwrap_or_default(),
                state_root: OpaqueHash::default(),
                reported,
            };
            self.history.push(new_block);

            // Truncate to maintain history size limit
            if self.history.len() > MAX_BLOCKS_HISTORY as usize {
                self.history.remove(0);
            }
        }
    }

    impl Mmr {
        /// Append a peak to the MMR.
        pub fn append(&mut self, peak: OpaqueHash) {
            self.peaks = mmr::append(self.peaks.clone(), peak);
        }

        /// Get the root of the MMR.
        pub fn root(&self) -> Option<OpaqueHash> {
            mmr::root(&self.peaks)
        }
    }
}
