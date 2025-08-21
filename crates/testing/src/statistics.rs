//! Statistics tests

use score::{
    extrinsic::{Extrinsic, ExtrinsicJson},
    safrole::{ValidatorDataJson, ValidatorsData},
    statistic::{Statistics, StatisticsJson},
    TimeSlot, ValidatorIndex,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

// FIXME: skipping the statistics tests since it's currently outdated.
//
// include!(concat!(env!("OUT_DIR"), "/statistics.rs"));

/// Run the statistics test
pub fn run(_test: &specjam::Test) -> anyhow::Result<()> {
    // let input = TestInput::from_json(&test.input)?;
    // let output = TestOutput::from_json(&test.output)?;
    //
    // // validate
    // let state = input.pre_state.statistics.update(
    //     input.input.slot,
    //     input.input.author_index,
    //     &input.input.extrinsic,
    // );
    // assert_eq!(state, output.post_state.statistics);
    Ok(())
}

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
    #[serde(flatten)]
    pub statistics: Statistics,

    /// The current time slot
    pub slot: TimeSlot,

    /// The current validators
    #[json(Vec<ValidatorDataJson>)]
    pub curr_validators: ValidatorsData,
}
