use crate::misc::*;

/// Represents a peak in the Merkle Mountain Range (MMR).
pub enum MmrPeak {
    None,             // Corresponds to [0] NULL
    Some(OpaqueHash), // Corresponds to [1] OpaqueHash
}

/// Represents the Merkle Mountain Range (MMR).
pub struct Mmr {
    pub peaks: Vec<MmrPeak>, // Sequence of MmrPeak
}

/// Represents a reported work package.
pub struct ReportedWorkPackage {
    pub hash: OpaqueHash,         // Corresponds to WorkReportHash
    pub exports_root: OpaqueHash, // Corresponds to ExportsRoot
}

/// Represents information about a block.
pub struct BlockInfo {
    pub header_hash: OpaqueHash,            // Corresponds to HeaderHash
    pub mmr: Mmr,                           // Corresponds to Mmr
    pub state_root: OpaqueHash,             // Corresponds to StateRoot
    pub reported: Vec<ReportedWorkPackage>, // Sequence of ReportedWorkPackage
}

/// Represents the history of blocks.
pub struct BlocksHistory {
    pub blocks: Vec<BlockInfo>, // Sequence of BlockInfo
}
