//! authorizations test

use runtime::tx;
use score::{CoreIndex, OpaqueHash, State, extrinsic::ReportGuarantee};
use serde::{Deserialize, Serialize};
use spacejson::Json;

include!(concat!(env!("OUT_DIR"), "/authorizations.rs"));

/// The authorizations STF `State` raw layout: `(auth-pools, auth-queues)`.
type RawState = (
    score::AuthorizationPools,
    score::Array<score::Array<OpaqueHash, { score::AUTH_QUEUE_SIZE }>, { score::CORES_COUNT }>,
);

impl From<RawState> for TestState {
    fn from((pools, queues): RawState) -> Self {
        TestState {
            auth_pools: pools.to_vec(),
            auth_queues: queues.iter().map(|q| q.to_vec()).collect(),
        }
    }
}

pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    // The authorizations STF `Output` is ASN.1 `NULL` (zero raw bytes).
    let (auths, pre, (), post) =
        codec::decode::<(Input, RawState, (), RawState)>(test.input.expect_bin()?)?;
    let input = TestInput { input: auths, pre_state: pre.into() };
    let output = TestOutput { post_state: post.into() };
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
    pub core: CoreIndex,
    #[json(hex)]
    pub auth_hash: OpaqueHash,
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
