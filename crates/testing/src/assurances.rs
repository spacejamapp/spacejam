#![cfg(test)]

use assurance::{
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
        assert_eq!(handler.post_state, self.post_state);
    }
}

crate::impl_assurances_tests! {
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
