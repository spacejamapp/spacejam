#![cfg(test)]

use codec::Json;
use paste::paste;
use score::{
    block::{Extrinsic, ExtrinsicJson},
    misc::{Ed25519Public, TimeSlot, ValidatorIndex},
};
use serde::{Deserialize, Serialize};
use stats::{State, StateJson};
use std::{fs, path::PathBuf};

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize)]
pub struct Input {
    slot: TimeSlot,
    author_index: ValidatorIndex,
    #[json(nested)]
    extrinsic: Extrinsic,
    #[json(Vec<String>)]
    reporters: Vec<Ed25519Public>,
}

#[derive(Json, Serialize, Deserialize, Debug)]
struct Test {
    #[json(nested)]
    pre_state: State,
    #[json(nested)]
    input: Input,
    output: (),
    #[json(nested)]
    post_state: State,
}

impl Test {
    fn run(self) -> anyhow::Result<()> {
        // TODO: compare results
        Ok(())
    }
}

#[allow(unused_macros)]
macro_rules! impl_history_tests {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<$name:snake>]() -> anyhow::Result<()> {
                let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                root.extend(["jamtestvectors", "statistics", "tiny"]);

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
        $(impl_history_tests!($name);)*
    };
}

impl_history_tests! {
    stats_with_empty_extrinsic_1,
    stats_with_epoch_change_1,
    stats_with_some_extrinsic_1
}
