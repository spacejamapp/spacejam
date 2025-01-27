//! This module contains the block information.

use crate::{
    block::{
        history::{Mmr, MmrJson},
        Block, Header,
    },
    extrinsic::Extrinsic,
    work::{ReportedWorkPackage, ReportedWorkPackageJson},
    OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents information about a block.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
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

impl BlockInfo {
    /// Mines a block
    pub fn mine(&self) -> anyhow::Result<Block> {
        let header = Header {
            parent: self.header_hash,
            parent_state_root: self.state_root,
            slot: crate::block::timeslot()?,
            ..Default::default()
        };

        // TODO: mine the transaction pool.
        let extrinsic = Extrinsic::default();
        Ok(Block { header, extrinsic })
    }
}
