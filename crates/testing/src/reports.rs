#![cfg(test)]

use score::runtime::tx::guarantee::{
    error::{Error, Result},
    State, StateJson,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
pub use types::*;

/// Test input.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct TestOutput {
    #[json(ResultJson<OutputJson, Error>)]
    pub output: Result<Output>,
    #[json(nested)]
    pub post_state: State,
}

// TODO: update the tests of the guarantee module
/* crate::impl_tests! {
    reports,
    @scale
    anchor_not_recent_1,
    bad_beefy_mmr_1,
    bad_code_hash_1,
    bad_core_index_1,
    bad_service_id_1,
    bad_signature_1,
    bad_state_root_1,
    bad_validator_index_1,
    core_engaged_1,
    dependency_missing_1,
    duplicate_package_in_recent_history_1,
    duplicated_package_in_report_1,
    future_report_slot_1,
    high_work_report_gas_1,
    many_dependencies_1,
    multiple_reports_1,
    no_enough_guarantees_1,
    not_authorized_1,
    not_authorized_2,
    not_sorted_guarantor_1,
    out_of_order_guarantees_1,
    report_before_last_rotation_1,
    report_curr_rotation_1,
    report_prev_rotation_1,
    reports_with_dependencies_1,
    reports_with_dependencies_2,
    reports_with_dependencies_3,
    reports_with_dependencies_4,
    reports_with_dependencies_5,
    reports_with_dependencies_6,
    segment_root_lookup_invalid_1,
    segment_root_lookup_invalid_2,
    service_item_gas_too_low_1,
    too_big_work_report_output_1,
    too_high_work_report_gas_1,
    too_many_dependencies_1,
    wrong_assignment_1
} */

mod types {
    use score::{
        extrinsic::{GuaranteesExtrinsic, ReportGuaranteeJson},
        service::{ReportedWorkPackage, ReportedWorkPackageJson},
        Block, Ed25519Public, TimeSlot,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    /// Input of the reporting module.
    #[derive(Debug, Clone, Serialize, Deserialize, Json)]
    pub struct Input {
        pub slot: TimeSlot,
        #[json(Vec<ReportGuaranteeJson>)]
        pub guarantees: GuaranteesExtrinsic,
    }

    impl From<Input> for Block {
        fn from(value: Input) -> Self {
            let mut block = Block::default();
            block.header.slot = value.slot;
            block.extrinsic.guarantees = value.guarantees;
            block
        }
    }

    /// Output of the reporting module.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct Output {
        #[json(nested)]
        pub reported: Vec<ReportedWorkPackage>,
        #[json(Vec<String>)]
        pub reporters: Vec<Ed25519Public>,
    }
}
