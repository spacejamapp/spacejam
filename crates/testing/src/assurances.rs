#![cfg(test)]

use assurance::{
    error::{Error, Result},
    state::{Input, InputJson, Output, OutputJson, State, StateJson},
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};

/// Test input for assurances
#[derive(Debug, Json, Serialize, Deserialize)]
pub struct TestInput {
    /// Input for the assurances
    #[json(nested)]
    pub input: Input,
    /// Pre-state for the assurances
    #[json(nested)]
    pub pre_state: State,
}

/// Test output for assurances
#[derive(Debug, Json, Serialize, Deserialize)]
pub struct TestOutput {
    /// Output for the assurances
    #[json(ResultJson<OutputJson, Error>)]
    pub output: Result<Output>,
    /// Post-state for the assurances
    #[json(nested)]
    pub post_state: State,
}

crate::impl_tests! {
    assurances,
    @scale
    assurance_for_not_engaged_core_1,
    assurance_with_bad_attestation_parent_1,
    assurances_for_stale_report_1,
    assurances_with_bad_signature_1,
    assurances_with_bad_validator_index_1,
    assurers_not_sorted_or_unique_1,
    assurers_not_sorted_or_unique_2,
    no_assurances_with_stale_report_1,
    no_assurances_1,
    some_assurances_1
}
