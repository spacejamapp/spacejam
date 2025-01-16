#![cfg(test)]

use core::result::Result;
use dispute::{error::Error, OffendersMark, OffendersMarkJson, State, StateJson};
use score::extrinsic::dispute::{DisputesExtrinsic, DisputesExtrinsicJson};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};

#[derive(Debug, Json, Serialize, Deserialize, Clone)]
pub struct Disputes {
    #[json(nested)]
    pub disputes: DisputesExtrinsic,
}

/// Test input.
#[derive(Debug, Json, Serialize, Deserialize)]
pub struct TestInput {
    #[json(nested)]
    pub input: Disputes,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Debug, Json, Serialize, Deserialize)]
pub struct TestOutput {
    #[json(ResultJson<OffendersMarkJson, Error>)]
    pub output: Result<OffendersMark, Error>,
    #[json(nested)]
    pub post_state: State,
}

crate::impl_tests! {
    disputes,
    @scale
    progress_invalidates_avail_assignments_1,
    progress_with_bad_signatures_1,
    progress_with_bad_signatures_2,
    progress_with_culprits_1,
    progress_with_culprits_2,
    progress_with_culprits_3,
    progress_with_culprits_4,
    progress_with_culprits_5,
    progress_with_culprits_6,
    progress_with_culprits_7,
    progress_with_faults_1,
    progress_with_faults_2,
    progress_with_faults_3,
    progress_with_faults_4,
    progress_with_faults_5,
    progress_with_faults_6,
    progress_with_faults_7,
    progress_with_no_verdicts_1,
    progress_with_verdict_signatures_from_previous_set_1,
    progress_with_verdict_signatures_from_previous_set_2,
    progress_with_verdicts_1,
    progress_with_verdicts_2,
    progress_with_verdicts_3,
    progress_with_verdicts_4,
    progress_with_verdicts_5,
    progress_with_verdicts_6
}
