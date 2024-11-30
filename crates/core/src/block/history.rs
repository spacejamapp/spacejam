use crate::misc::*;
use json::Json;
use scale::{Decode, Encode};

/// Represents a peak in the Merkle Mountain Range (MMR).
pub type MmrPeak = Option<OpaqueHash>;

/// Represents the Merkle Mountain Range (MMR).
#[derive(Debug, Encode, Decode)]
pub struct Mmr {
    pub peaks: Vec<MmrPeak>,
}

/// Represents a reported work package.
#[derive(Debug, Encode, Decode, Json)]
pub struct ReportedWorkPackage {
    #[json(hex)]
    pub hash: OpaqueHash,
    #[json(hex)]
    pub exports_root: OpaqueHash,
}

/// Represents information about a block.
#[derive(Debug, Encode, Decode)]
pub struct BlockInfo {
    pub header_hash: OpaqueHash,
    pub mmr: Mmr,
    pub state_root: OpaqueHash,
    pub reported: Vec<ReportedWorkPackage>,
}

/// Represents the history of blocks.
#[derive(Debug, Encode, Decode)]
pub struct BlocksHistory {
    pub blocks: Vec<BlockInfo>,
}
