//! history test

use anyhow::Result;
use runtime::tx::block::history;
use score::{
    OpaqueHash,
    block::{History, HistoryJson},
    service::{ReportedWorkPackage, ReportedWorkPackageJson},
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

include!(concat!(env!("OUT_DIR"), "/history.rs"));

/// Run the history test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let input = TestInput::from_json(&test.input)?;
    let output = TestOutput::from_json(&test.output)?;
    let mut history = input.pre_state.beta.clone();
    if let Some(last) = history.history.last_mut() {
        last.state_root = input.input.parent_state_root;
    }
    history::import(
        &mut history,
        input.input.header_hash,
        input.input.accumulate_root,
        input.input.work_packages.clone(),
    );
    assert_eq!(output.post_state.beta, history);
    Ok(())
}

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
    pub beta: History,
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
