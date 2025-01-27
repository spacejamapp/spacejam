//! This module contains the block and its related structures.

use std::time::{Duration, UNIX_EPOCH};

use crate::{extrinsic::*, HeaderHash, TimeSlot};
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    header::{Header, HeaderJson},
    history::History,
    info::{BlockInfo, BlockInfoJson},
};

pub mod header;
pub mod history;
mod info;

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
    /// Returns the hash of the block
    pub fn hash(&self) -> anyhow::Result<HeaderHash> {
        let encoded = codec::encode(&self.header)?;
        Ok(crypto::blake2b(&encoded))
    }
}

/// Returns the current timeslot
pub fn timeslot() -> anyhow::Result<TimeSlot> {
    let era = Duration::from_secs(crate::JAM_COMMON_ERA_AFTER_UNIX_EPOCH as u64);
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH + era)?
        .as_secs() as u32;

    Ok(now / 6)
}
