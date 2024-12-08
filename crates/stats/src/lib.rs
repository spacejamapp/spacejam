use codec::Json;
use score::{
    block::Extrinsic,
    misc::{Ed25519Public, TimeSlot, ValidatorDataJson, ValidatorIndex, ValidatorsData},
    stats::*,
};
use serde::{Deserialize, Serialize};

/// State of the stats
///
/// NOTE: this should be moved to storage in the future
#[derive(Json, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct State {
    #[json(nested)]
    pub pi: Statistics,
    pub tau: TimeSlot,
    #[json(Vec<ValidatorDataJson>)]
    pub kappa_prime: ValidatorsData,
}

/// Registry of the stats
pub struct Stats {
    pub state: State,
    pub next_state: State,
}

impl Stats {
    pub fn update(
        self,
        _slot: TimeSlot,
        _author_index: ValidatorIndex,
        _extrinsic: Extrinsic,
        _reportors: Vec<Ed25519Public>,
    ) -> Self {
        // TODO: implement
        self
    }
}
