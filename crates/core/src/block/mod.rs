//! This module contains the block and its related structures.

use crate::{
    extrinsic::*,
    work::{ReportedWorkPackage, ReportedWorkPackageJson},
    HeaderHash, OpaqueHash, TimeSlot,
};
use history::{Mmr, MmrJson};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::time::{Duration, UNIX_EPOCH};
pub use {
    builder::Builder,
    header::{Header, HeaderJson},
    history::History,
};

mod builder;
pub mod header;
pub mod history;

/// Represents a block in the system.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Default, Clone)]
pub struct Block {
    /// The header of the block
    #[json(nested)]
    pub header: Header,
    /// The extrinsic of the block
    #[json(nested)]
    pub extrinsic: Extrinsic,
}

impl Block {
    /// Returns a builder for the block
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the hash of the block
    pub fn hash(&self) -> anyhow::Result<HeaderHash> {
        let encoded = codec::encode(&self.header)?;
        Ok(crypto::blake2b(&encoded))
    }
}

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

impl From<Header> for BlockInfo {
    fn from(header: Header) -> Self {
        Self {
            header_hash: header.hash().unwrap(),
            mmr: Mmr::default(),
            state_root: header.parent_state_root,
            reported: vec![],
        }
    }
}

/// Returns the current timeslot
pub fn timeslot() -> anyhow::Result<TimeSlot> {
    let era = Duration::from_secs(crate::JAM_COMMON_ERA_AFTER_UNIX_EPOCH as u64);
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH + era)?
        .as_secs() as u32;

    Ok(now / crate::SLOT_PERIOD)
}
