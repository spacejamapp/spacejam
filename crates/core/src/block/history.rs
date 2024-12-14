use crate::misc::*;
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
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct BlocksHistory {
    #[json(nested)]
    pub blocks: Vec<BlockInfo>,
}
