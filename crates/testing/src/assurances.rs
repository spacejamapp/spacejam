#![cfg(test)]

use assurance::{
    error::{Error, Result},
    state::*,
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

mod types {
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
}
