//! history test

use runtime::tx::block::history;
use score::{OpaqueHash, block::History, service::ReportedWorkPackage};
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/history.rs"));

/// Run the history test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let (input, pre_state, (), post_state) =
        codec::decode::<(Input, State, (), State)>(test.input.expect_bin()?)?;
    let input = TestInput { input, pre_state };
    let output = TestOutput {
        output: None,
        post_state,
    };
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub header_hash: OpaqueHash,
    pub parent_state_root: OpaqueHash,
    pub accumulate_root: OpaqueHash,
    pub work_packages: Vec<ReportedWorkPackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub beta: History,
}

/// Test input for history
#[derive(Serialize, Deserialize, Debug)]
pub struct TestInput {
    pub input: Input,
    pub pre_state: State,
}

/// Test output for history
#[derive(Serialize, Deserialize, Debug)]
pub struct TestOutput {
    pub output: Option<()>,
    pub post_state: State,
}
