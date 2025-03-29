//! Statistics tests

use score::{
    extrinsic::{Extrinsic, ExtrinsicJson},
    safrole::{ValidatorDataJson, ValidatorsData},
    statistic::{Statistics, StatisticsJson},
    TimeSlot, ValidatorIndex,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize)]
pub struct Input {
    pub slot: TimeSlot,
    pub author_index: ValidatorIndex,
    #[json(nested)]
    pub extrinsic: Extrinsic,
}

/// Test input.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Deserialize, Serialize, Json, Debug)]
pub struct TestOutput {
    #[json(nested)]
    pub post_state: State,
}

/// State of the stats
///
/// NOTE: this should be moved to storage in the future
#[derive(Json, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct State {
    /// The statistics state
    #[json(nested)]
    pub statistics: Statistics,

    /// The current time slot
    pub slot: TimeSlot,

    /// The current validators
    #[json(Vec<ValidatorDataJson>)]
    pub curr_validators: ValidatorsData,
}

crate::impl_tests! {
    statistics,
    @scale
    stats_with_empty_extrinsic_1,
    stats_with_epoch_change_1,
    stats_with_some_extrinsic_1
}
