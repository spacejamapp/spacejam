//! Block history

use crate::{service::ReportedWorkPackage, OpaqueHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a peak in the Merkle Mountain Range (MMR).
pub type MmrPeak = Option<OpaqueHash>;

/// Block history extension trait
pub trait History {
    /// Import a new block into the chain according to graypaper section 7.1-7.4.
    fn import(
        &mut self,
        header_hash: OpaqueHash,
        state_root: OpaqueHash,
        accumulated_root: OpaqueHash,
        reported: Vec<ReportedWorkPackage>,
    );
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

    impl History for Vec<BlockInfo> {
        fn import(
            &mut self,
            header_hash: OpaqueHash,
            state_root: OpaqueHash,
            accumulated_root: OpaqueHash,
            reported: Vec<ReportedWorkPackage>,
        ) {
            let Some(last) = self.last_mut() else {
                let new_block = BlockInfo {
                    header_hash,
                    mmr: Mmr {
                        peaks: vec![Some(accumulated_root)],
                    },
                    state_root: OpaqueHash::default(),
                    reported,
                };
                self.push(new_block.clone());
                return;
            };

            // Update the state root of the parent block if it exists
            last.state_root = state_root;
            let mut mmr = last.mmr.clone();
            mmr.append(accumulated_root);

            // Append the new block to history
            let new_block = BlockInfo {
                header_hash,
                state_root: OpaqueHash::default(),
                mmr,
                reported,
            };
            self.push(new_block);

            // Truncate to maintain history size limit
            if self.len() > MAX_BLOCKS_HISTORY as usize {
                self.remove(0);
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
