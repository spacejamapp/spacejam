#![cfg(test)]

use paste::paste;
use score::{
    block::{BlockInfo, BlockInfoJson, BlocksHistory},
    work::{ReportedWorkPackage, ReportedWorkPackageJson},
    OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::{fs, path::PathBuf};

use crate::init_tracing;

#[derive(Serialize, Deserialize, Json, Debug)]
pub struct Input {
    #[json(hex)]
    header_hash: OpaqueHash,
    #[json(hex)]
    parent_state_root: OpaqueHash,
    #[json(hex)]
    accumulate_root: OpaqueHash,
    #[json(nested)]
    work_packages: Vec<ReportedWorkPackage>,
}

#[derive(Serialize, Deserialize, Json, Debug, Clone)]
pub struct State {
    #[json(nested)]
    beta: Vec<BlockInfo>,
}

#[derive(Serialize, Deserialize, Json, Debug)]
pub struct Test {
    #[json(nested)]
    input: Input,
    #[json(nested)]
    pre_state: State,
    #[json(Option<()>)]
    output: Option<()>,
    #[json(nested)]
    post_state: State,
}

impl Test {
    pub fn run(&self) -> anyhow::Result<()> {
        init_tracing();
        let state = self.pre_state.clone();
        let mut history = BlocksHistory { blocks: state.beta };
        history.import(
            self.input.header_hash,
            self.input.parent_state_root,
            self.input.accumulate_root,
            self.input.work_packages.clone(),
        );

        assert_eq!(self.post_state.beta, history.blocks);
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
                root.extend(["jamtestvectors", "history", "data"]);

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
    progress_blocks_history_1,
    progress_blocks_history_2,
    progress_blocks_history_3,
    progress_blocks_history_4
}
