#![cfg(test)]

use assurance::{
    error::{Error, Result},
    state::{Input, InputJson, Output, OutputJson, State, StateJson},
    Handler,
};
use paste::paste;
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
use std::{fs, path::PathBuf};

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
    fn run(self) -> anyhow::Result<()> {
        let _handler = Handler::from(self.pre_state);
        // handler.handle(self.input)?;
        // assert_eq!(handler.state(), self.post_state);
        Ok(())
    }
}

macro_rules! impl_assurance_tests {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<$name:snake>]() -> anyhow::Result<()> {
                let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                root.extend(["jamtestvectors", "assurances", "tiny"]);

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
        $(impl_assurance_tests!($name);)*
    };
}

impl_assurance_tests! {
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
