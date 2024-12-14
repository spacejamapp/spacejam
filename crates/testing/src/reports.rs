#![cfg(test)]

use report::{
    error::{Error, Result},
    state::{Input, InputJson, Output, OutputJson, State, StateJson},
    Handler,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};

#[derive(Debug, Clone, Serialize, Deserialize, Json)]
struct Test {
    #[json(nested)]
    input: Input,
    #[json(ResultJson<OutputJson, Error>)]
    output: Result<Output>,
    #[json(nested)]
    pre_state: State,
    #[json(nested)]
    post_state: State,
}

impl Test {
    fn run(self) {
        let mut handler = Handler::from(self.pre_state);
        let output = handler.handle(self.input);
        assert_eq!(output, self.output);
        assert_eq!(
            handler.next.auth_pools, self.post_state.auth_pools,
            "auth_pools"
        );
        assert_eq!(
            handler.next.avail_assignments, self.post_state.avail_assignments,
            "avail_assignments"
        );
        assert_eq!(
            handler.next.curr_validators, self.post_state.curr_validators,
            "curr_validators"
        );
        assert_eq!(
            handler.next.prev_validators, self.post_state.prev_validators,
            "prev_validators"
        );
        assert_eq!(handler.next.entropy, self.post_state.entropy, "entropy");
        assert_eq!(handler.next.services, self.post_state.services, "services");
        assert_eq!(
            handler.next.offenders, self.post_state.offenders,
            "offenders"
        );
    }
}

crate::impl_reports_tests! {
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
}
