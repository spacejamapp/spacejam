#![cfg(test)]

use core::result::Result;
use score::{
    extrinsic::dispute::{
        DisputesExtrinsic, DisputesExtrinsicJson, DisputesRecords, DisputesRecordsJson,
    },
    runtime::tx::dispute::error::Error,
    safrole::{ValidatorDataJson, ValidatorsData},
    service::{AvailabilityAssignmentJson, AvailabilityAssignments},
    Ed25519Public, TimeSlot,
};
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

#[derive(Json, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct OffendersMark {
    /// [H_o] Offenders marker
    #[json(Vec<String>)]
    pub offenders_mark: Vec<Ed25519Public>,
}

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize, Clone)]
pub struct State {
    /// [ψ] Disputes verdicts and offenders
    #[json(nested)]
    pub psi: DisputesRecords,
    /// [ρ] Availability cores assignments
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub rho: AvailabilityAssignments,
    /// [τ] Timeslot
    pub tau: TimeSlot,
    /// [κ] Validators active in the current epoch
    #[json(Vec<ValidatorDataJson>)]
    pub kappa: ValidatorsData,
    /// [λ] Validators active in the previous epoch
    #[json(Vec<ValidatorDataJson>)]
    pub lambda: ValidatorsData,
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
