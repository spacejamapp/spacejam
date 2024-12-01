use crate::misc::*;
use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents a peak in the Merkle Mountain Range (MMR).
pub type MmrPeak = Option<OpaqueHash>;

/// Represents the Merkle Mountain Range (MMR).
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Mmr {
    #[json(Vec<Option<String>>)]
    pub peaks: Vec<MmrPeak>,
}

/// Represents a reported work package.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct ReportedWorkPackage {
    #[json(hex)]
    pub hash: OpaqueHash,
    #[json(hex)]
    pub exports_root: OpaqueHash,
}

/// Represents information about a block.
#[derive(Debug, Serialize, Deserialize, Json)]
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
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct BlocksHistory {
    #[json(nested)]
    pub blocks: Vec<BlockInfo>,
}
