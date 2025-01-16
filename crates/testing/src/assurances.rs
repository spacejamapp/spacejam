//! This module contains the tests for the assurance module.

use assurance::{Error, Result, State, StateJson};
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
        work::report::{WorkReport, WorkReportJson},
        HeaderHash, TimeSlot,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

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

    impl From<Input> for score::Block {
        fn from(input: Input) -> Self {
            let mut block = score::Block::default();
            block.header.slot = input.slot;
            block.header.parent = input.parent;
            block.extrinsic.assurances = input.assurances;
            block
        }
    }

    /// The output of the assurance module.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct Output {
        #[json(Vec<WorkReportJson>)]
        pub reported: Vec<WorkReport>,
    }
}
