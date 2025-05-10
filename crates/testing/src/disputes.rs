#![cfg(test)]

use core::result::Result;
use runtime::tx::dispute::error::Error;
use score::{
    extrinsic::dispute::{
        DisputesExtrinsic, DisputesExtrinsicJson, DisputesRecords, DisputesRecordsJson,
    },
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

include!(concat!(env!("OUT_DIR"), "/disputes.rs"));
