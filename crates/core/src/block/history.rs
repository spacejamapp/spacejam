//! Block history

use crate::{
    OpaqueHash,
    block::{BlockInfo, BlockInfoJson},
};
use anyhow::Result;
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

impl History {
    /// Complete the state root of the last block in the history
    pub fn complete_state_root(&mut self, state_root: OpaqueHash) -> Result<Option<OpaqueHash>> {
        let Some(last) = self.history.last_mut() else {
            return Ok(None);
        };

        last.state_root = state_root;
        Ok(Some(last.header_hash))
    }
}

/// Represents the Merkle Mountain Range (MMR).
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct Mmr {
    #[json(Vec<Option<String>>)]
    pub peaks: Vec<MmrPeak>,
}
