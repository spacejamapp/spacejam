//! This module contains the tests for the assurance module.

use runtime::tx::assurance::{Error, Result};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
use types::*;

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

include!(concat!(env!("OUT_DIR"), "/assurances.rs"));

mod types {
    use score::{
        extrinsic::{AssurancesExtrinsic, AvailAssuranceJson},
        safrole::{ValidatorDataJson, ValidatorsData},
        service::{
            AvailabilityAssignmentJson, AvailabilityAssignments, WorkReport, WorkReportJson,
        },
        HeaderHash, TimeSlot,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    /// The state of the assurance module.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct State {
        /// (ρ†) rho dagger, which is the pending reports (ϱ) after that any
        /// work report judged as uncertain or invalid has been removed from it.
        /// On success, mutated to get [ϱ‡].
        #[json(Vec<Option<AvailabilityAssignmentJson>>)]
        pub avail_assignments: AvailabilityAssignments,
        /// (κ') posterior active validators.
        #[json(Vec<ValidatorDataJson>)]
        pub curr_validators: ValidatorsData,
    }

    /// The input to the assurance module.
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

    /// The output of the assurance module.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct Output {
        #[json(Vec<WorkReportJson>)]
        pub reported: Vec<WorkReport>,
    }
}
