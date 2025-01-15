#![cfg(test)]

use score::{
    block::{BlockInfo, BlockInfoJson},
    work::{ReportedWorkPackage, ReportedWorkPackageJson},
    OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Serialize, Deserialize, Json, Debug)]
pub struct Input {
    #[json(hex)]
    pub header_hash: OpaqueHash,
    #[json(hex)]
    pub parent_state_root: OpaqueHash,
    #[json(hex)]
    pub accumulate_root: OpaqueHash,
    #[json(nested)]
    pub work_packages: Vec<ReportedWorkPackage>,
}

#[derive(Serialize, Deserialize, Json, Debug, Clone)]
pub struct State {
    #[json(nested)]
    pub beta: Vec<BlockInfo>,
}

/// Test input for history
#[derive(Serialize, Deserialize, Json, Debug)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output for history
#[derive(Serialize, Deserialize, Json, Debug)]
pub struct TestOutput {
    #[json(Option<()>)]
    pub output: Option<()>,
    #[json(nested)]
    pub post_state: State,
}

crate::impl_tests! {
    history,
    progress_blocks_history_1,
    progress_blocks_history_2,
    progress_blocks_history_3,
    progress_blocks_history_4
}
