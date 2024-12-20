use crate::{misc::*, MAX_BLOCKS_HISTORY};
use merkle::mmr;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a peak in the Merkle Mountain Range (MMR).
pub type MmrPeak = Option<OpaqueHash>;

/// Represents the Merkle Mountain Range (MMR).
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct Mmr {
    #[json(Vec<Option<String>>)]
    pub peaks: Vec<MmrPeak>,
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

/// Represents a reported work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct ReportedWorkPackage {
    #[json(hex)]
    pub hash: OpaqueHash,
    #[json(hex)]
    pub exports_root: OpaqueHash,
}

/// Represents information about a block.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct BlockInfo {
    #[json(hex)]
    pub header_hash: OpaqueHash,
    #[json(nested)]
    pub mmr: Mmr,
    #[json(hex)]
    pub state_root: OpaqueHash,
    #[json(nested)]
    pub reported: Vec<ReportedWorkPackage>,
}

/// Represents the history of blocks.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct BlocksHistory {
    #[json(nested)]
    pub blocks: Vec<BlockInfo>,
}

impl BlocksHistory {
    /// Import a new block into the chain according to graypaper section 7.1-7.4.
    pub fn import(
        &mut self,
        header_hash: OpaqueHash,
        state_root: OpaqueHash,
        accumulated_root: OpaqueHash,
        reported: Vec<ReportedWorkPackage>,
    ) {
        let Some(last) = self.blocks.last_mut() else {
            self.blocks.push(BlockInfo {
                header_hash,
                mmr: Mmr {
                    peaks: vec![Some(accumulated_root)],
                },
                state_root: OpaqueHash::default(),
                reported,
            });
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
        self.blocks.push(new_block);

        // Truncate to maintain history size limit
        if self.blocks.len() > MAX_BLOCKS_HISTORY {
            self.blocks.remove(0);
        }
    }
}
