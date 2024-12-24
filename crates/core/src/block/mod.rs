use crate::{extrinsic::*, HeaderHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    header::{Header, HeaderJson},
    history::BlocksHistory,
};

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
    /// Returns the hash of the block
    pub fn hash(&self) -> anyhow::Result<HeaderHash> {
        let encoded = codec::encode(&self.header)?;
        Ok(crypto::blake2b(&encoded))
    }
}
