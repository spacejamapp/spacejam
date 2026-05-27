//! authorizations test

use runtime::tx;
use score::{CoreIndex, OpaqueHash, State, extrinsic::ReportGuarantee};
use serde::{Deserialize, Serialize};
use spacejson::Json;

// FIXME: the ordering of the authorization pools could be wrong in the test cases,
// note that we follow the result in the tests of traces.
//
// include!(concat!(env!("OUT_DIR"), "/authorizations.rs"));

pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let input = TestInput::from_json(test.input.expect_json()?)?;
    let output = TestOutput::from_json(test.output.expect_json()?)?;
    let state: score::State = input.pre_state.clone().into();
    let post: score::State = output.post_state.clone().into();

    // Validate post state
    let result = tx::guarantee::pools(
        input.input.slot,
        &state.pools,
        &state.authorization,
        &input.input.auths.into_iter().map(|a| a.into()).collect(),
    );

    assert_eq!(result, post.pools);
    assert_eq!(state.authorization, post.authorization);
    Ok(())
}

/// Test state for authorizations
#[derive(Serialize, Deserialize, Json, Debug, Clone)]
pub struct TestState {
    #[json(Vec<Vec<String>>)]
    pub auth_pools: Vec<Vec<OpaqueHash>>,
    #[json(Vec<Vec<String>>)]
    pub auth_queues: Vec<Vec<OpaqueHash>>,
}

impl From<TestState> for State {
    fn from(state: TestState) -> Self {
        let mut authorization: score::Array<
            score::Array<score::OpaqueHash, { score::AUTH_QUEUE_SIZE }>,
            { score::CORES_COUNT },
        > = Default::default();
        for (core, queue) in state.auth_queues.iter().enumerate() {
            for (slot, hash) in queue.iter().enumerate() {
                authorization[core][slot] = *hash;
            }
        }
        Self {
            pools: state.auth_pools.try_into().expect("invalid auth pools"),
            authorization,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Json, Debug, Clone)]
pub struct Authorization {
    #[json(hex)]
    pub auth_hash: OpaqueHash,
    pub core: CoreIndex,
}

impl From<Authorization> for ReportGuarantee {
    fn from(auth: Authorization) -> Self {
        let mut guarantee = ReportGuarantee::default();
        guarantee.report.authorizer_hash = auth.auth_hash;
        guarantee.report.core_index = auth.core;
        guarantee
    }
}

#[derive(Serialize, Deserialize, Json, Debug, Clone)]
pub struct Input {
    pub slot: u32,
    #[json(nested)]
    pub auths: Vec<Authorization>,
}

/// Test input for authorizations
#[derive(Serialize, Deserialize, Json, Debug)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: TestState,
}

/// Test output for authorizations
#[derive(Serialize, Deserialize, Json, Debug)]
pub struct TestOutput {
    #[json(nested)]
    pub post_state: TestState,
}
