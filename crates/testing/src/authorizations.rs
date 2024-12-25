//! aa

use score::{extrinsic::ReportGuarantee, CoreIndex, OpaqueHash, State};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Test state for authorizations
#[derive(Serialize, Deserialize, Json, Debug)]
pub struct TestState {
    #[json(Vec<Vec<String>>)]
    pub auth_pools: Vec<Vec<OpaqueHash>>,
    #[json(Vec<Vec<String>>)]
    pub auth_queues: Vec<Vec<OpaqueHash>>,
}

impl From<TestState> for State {
    fn from(state: TestState) -> Self {
        Self {
            pools: state.auth_pools.try_into().expect("invalid auth pools"),
            authorization: state.auth_queues.try_into().expect("invalid auth queues"),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Json, Debug)]
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

#[derive(Serialize, Deserialize, Json, Debug)]
pub struct Input {
    pub slot: u32,
    #[json(nested)]
    pub auths: Vec<Authorization>,
}

#[derive(Serialize, Deserialize, Json, Debug)]
pub struct Test {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: TestState,
    #[json(nested)]
    pub post_state: TestState,
}

impl Test {
    pub fn run(self) {
        let pre: State = self.pre_state.into();
        let post: State = self.post_state.into();
        let result = report::auth::handle(
            pre,
            self.input.slot,
            self.input.auths.into_iter().map(|a| a.into()).collect(),
        )
        .expect("failed to handle authorizations");
        assert_eq!(result, post);
    }
}

crate::impl_authorizations_tests!(
    progress_authorizations_1,
    progress_authorizations_2,
    progress_authorizations_3
);
