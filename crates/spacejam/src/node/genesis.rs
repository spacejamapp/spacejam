//! The configuration of SpaceJam

use score::{
    block::{Block, BlockJson},
    safrole::ValidatorDataJson,
    Entropy,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// The genesis configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Genesis {
    /// The genesis block
    pub block: BlockJson,

    /// The current validators
    pub validators: Vec<ValidatorDataJson>,

    /// The entropy
    pub entropy: [String; 4],
}

impl Default for Genesis {
    fn default() -> Self {
        let empty = format!("0x{}", hex::encode(Entropy::default()));
        Self {
            block: Block::default().to_json(),
            validators: vec![],
            entropy: [empty.clone(), empty.clone(), empty.clone(), empty],
        }
    }
}
