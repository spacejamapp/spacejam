//! Statistics tests

use score::{
    extrinsic::{Extrinsic, ExtrinsicJson},
    TimeSlot, ValidatorIndex,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use statistic::{State, StateJson};

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

crate::impl_tests! {
    statistics,
    @scale
    stats_with_empty_extrinsic_1,
    stats_with_epoch_change_1,
    stats_with_some_extrinsic_1
}
