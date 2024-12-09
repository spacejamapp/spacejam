#![cfg(test)]

use codec::{Json, ResultJson};
use core::result::Result;
use dispute::{error::Error, DisputesHandler, OffendersMark, OffendersMarkJson, State, StateJson};
use paste::paste;
use score::dispute::{DisputesExtrinsic, DisputesExtrinsicJson};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::init_tracing;

#[derive(Debug, Json, Serialize, Deserialize, Clone)]
pub struct Disputes {
    #[json(nested)]
    disputes: DisputesExtrinsic,
}

#[derive(Debug, Json, Serialize, Deserialize)]
pub struct Test {
    #[json(nested)]
    pub input: Disputes,
    #[json(nested)]
    pub pre_state: State,
    #[json(ResultJson<OffendersMarkJson, Error>)]
    pub output: Result<OffendersMark, Error>,
    #[json(nested)]
    pub post_state: State,
}

impl Test {
    pub fn run(&mut self) -> anyhow::Result<()> {
        init_tracing();
        let mut handler = DisputesHandler::from(self.pre_state.clone());
        let output = handler.handle(self.input.disputes.clone());

        assert_eq!(output, self.output, "output mismatch");
        assert_eq!(handler.next_state.psi, self.post_state.psi, "psi mismatch");
        assert_eq!(handler.next_state.rho, self.post_state.rho, "rho mismatch");
        assert_eq!(handler.next_state.tau, self.post_state.tau, "tau mismatch");
        assert_eq!(
            handler.next_state.kappa, self.post_state.kappa,
            "kappa mismatch"
        );
        assert_eq!(
            handler.next_state.lambda, self.post_state.lambda,
            "lambda mismatch"
        );
        Ok(())
    }
}

#[allow(unused_macros)]
macro_rules! impl_disputes_tests {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<$name:snake>]() -> anyhow::Result<()> {
                let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                root.extend(["jamtestvectors", "disputes", "tiny"]);

                let pattern = stringify!($name).split("_").collect::<Vec<&str>>();
                let mut name = pattern[..pattern.len() - 1].join("_");
                name.push_str(&format!(
                    "-{}",
                    pattern.last().expect("pattern must have at least one element")
                ));

                root.push(name);
                root.set_extension("json");

                let json = fs::read_to_string(root)?;
                Test::from_json(&json)?.run()
            }
        }
    };
    ($($name:ident),*) => {
        $(impl_disputes_tests!($name);)*
    };
}

impl_disputes_tests! {
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
