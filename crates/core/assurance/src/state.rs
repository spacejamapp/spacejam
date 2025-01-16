use score::{
    extrinsic::{AssurancesExtrinsic, AvailAssuranceJson},
    validator::{ValidatorDataJson, ValidatorsData},
    work::{
        report::{WorkReport, WorkReportJson},
        AvailabilityAssignmentJson, AvailabilityAssignments,
    },
    HeaderHash, TimeSlot,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct State {
    /// [ρ†] rho dagger, which is the pending reports (ϱ) after that any
    /// work report judged as uncertain or invalid has been removed from it.
    /// On success, mutated to get [ϱ‡].
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub avail_assignments: AvailabilityAssignments,
    /// [κ'] Posterior active validators.
    #[json(Vec<ValidatorDataJson>)]
    pub curr_validators: ValidatorsData,
    /// [ϱ‡] The reports that have been judged as available.
    #[serde(default)]
    #[json(Vec<WorkReportJson>)]
    pub reported: Vec<WorkReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct Input {
    /// [E_A] Assurances extrinsic.
    #[json(Vec<AvailAssuranceJson>)]
    pub assurances: AssurancesExtrinsic,

    /// [H_t] Block's timeslot.
    pub slot: TimeSlot,

    /// [H_p] Parent hash.
    #[json(hex)]
    pub parent: HeaderHash,
}

#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Output {
    #[json(Vec<WorkReportJson>)]
    pub reported: Vec<WorkReport>,
}
