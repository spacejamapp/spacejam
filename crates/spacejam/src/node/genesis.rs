//! The configuration of SpaceJam

use runtime::Validator;
use score::{
    block::{header::EValidator, Block, BlockJson},
    safrole::{ValidatorData, ValidatorDataJson},
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

use crate::validator::LocalValidator;

/// The genesis configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Genesis {
    /// The genesis block
    pub block: BlockJson,

    /// The current validators
    pub validators: Vec<ValidatorDataJson>,
}

impl Genesis {
    /// Create a new genesis configuration
    pub fn new(validators: Vec<ValidatorData>) -> Self {
        let mut ekeys = [EValidator::default(); score::VALIDATORS_COUNT as usize];
        for (i, v) in validators.iter().enumerate() {
            ekeys[i as usize] = EValidator {
                bandersnatch: v.bandersnatch,
                ed25519: v.ed25519,
            };
        }

        Self {
            block: Block::genesis(ekeys).to_json(),
            validators: validators.into_iter().map(|v| v.to_json()).collect(),
        }
    }
}

impl Default for Genesis {
    fn default() -> Self {
        let validators = (0..score::VALIDATORS_COUNT)
            .map(|i| LocalValidator::from([i as u8; 32]).data())
            .collect::<Vec<_>>();

        Self::new(validators)
    }
}
