//! The configuration of SpaceJam

use serde::{Deserialize, Serialize};
use spacejson::Json;

use crate::{
    block::{Block, BlockJson},
    state::Storage,
    validator::{Validator, ValidatorDataJson},
};

/// The configuration of SpaceJam
pub trait Config {
    /// The validator of SpaceJam
    type Validator: Validator + Default + From<[u8; 32]>;

    /// The database of SpaceJam
    type Db: Storage;
}

/// The genesis configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Genesis {
    /// The genesis block
    pub block: BlockJson,

    /// The current validators
    pub validators: Vec<ValidatorDataJson>,
}

impl Default for Genesis {
    fn default() -> Self {
        Self {
            block: Block::default().to_json(),
            validators: vec![],
        }
    }
}
