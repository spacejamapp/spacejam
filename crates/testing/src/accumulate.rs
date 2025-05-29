//! Accumulate tests

use score::{
    service::{
        AccumulatedQueue, Privileges, ReadyQueue, ReadyReportJson, ServiceAccount, ServiceItem,
        ServiceItemJson, WorkReport, WorkReportJson,
    },
    Entropy, Gas, OpaqueHash, ServiceId, TimeSlot,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
use std::collections::BTreeMap;

/// Accumulate test
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Test {
    /// The input
    #[json(nested)]
    pub input: TestInput,

    /// The output
    #[json(ResultJson<String, ()>)]
    pub output: Result<OpaqueHash, ()>,
}

/// Test input
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Input {
    /// The time slot from the block header
    pub slot: TimeSlot,

    /// The reports
    #[json(nested)]
    pub reports: Vec<WorkReport>,
}

/// Test input
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestInput {
    /// The input
    #[json(nested)]
    pub input: Input,

    /// The pre-state
    #[json(nested)]
    pub pre_state: State,
}

/// Test output
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestOutput {
    /// The post-state
    #[json(nested)]
    pub post_state: State,

    /// The output
    #[json(ResultJson<String, ()>)]
    pub output: Result<OpaqueHash, ()>,
}

/// State for the accumulation
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct State {
    /// The time slot
    pub slot: TimeSlot,

    /// The current entropy
    #[json(hex)]
    pub entropy: Entropy,

    /// The ready queue
    #[json(Vec<Vec<ReadyReportJson>>)]
    pub ready_queue: ReadyQueue,

    /// The accumulated reports
    #[json(Vec<Vec<String>>)]
    pub accumulated: AccumulatedQueue,

    /// The privileges
    #[json(nested)]
    pub privileges: PrivilegesWrap,

    /// The accounts
    #[json(nested)]
    pub accounts: Vec<ServiceItem>,
}

impl State {
    /// Get the accounts
    pub fn accounts(&self) -> BTreeMap<u32, ServiceAccount> {
        self.accounts
            .iter()
            .map(|item| (item.id, item.data.clone().into()))
            .collect()
    }
}

/// Privileges wrapper
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct PrivilegesWrap {
    /// The bless service id
    pub bless: ServiceId,

    /// The designate service id
    pub designate: ServiceId,

    /// The assign service id
    pub assign: ServiceId,

    /// The always accumulate service ids
    #[json(nested)]
    pub always_acc: Vec<AlwaysAccumulateMapItem>,
}

impl From<PrivilegesWrap> for Privileges {
    fn from(value: PrivilegesWrap) -> Self {
        Privileges {
            bless: value.bless,
            designate: value.designate,
            assign: value.assign,
            always_acc: value
                .always_acc
                .into_iter()
                .map(|item| (item.service, item.gas))
                .collect(),
        }
    }
}

impl From<Privileges> for PrivilegesWrap {
    fn from(value: Privileges) -> Self {
        PrivilegesWrap {
            bless: value.bless,
            designate: value.designate,
            assign: value.assign,
            always_acc: value
                .always_acc
                .into_iter()
                .map(|(service, gas)| AlwaysAccumulateMapItem { service, gas })
                .collect(),
        }
    }
}
/// Always accumulate service id
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct AlwaysAccumulateMapItem {
    /// The service id
    pub service: ServiceId,

    /// The gas
    pub gas: Gas,
}

include!(concat!(env!("OUT_DIR"), "/accumulate.rs"));

mod local {
    use super::*;
    use specjam::Registry;
    use tracing_subscriber::EnvFilter;

    #[test]
    fn test_parse_service() {
        tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        let registry = Registry::new("../../res/jam-test-vectors");
        let test = registry
            .accumulate(specjam::Scale::Tiny)
            .unwrap()
            .test("process_one_immediate_report-1")
            .unwrap();

        let input = TestInput::from_json(&test.input).unwrap();
        // let output = TestOutput::from_json(&test.output).unwrap();
        // let accounts = input.pre_state.accounts();

        let blob = &input.pre_state.accounts[0].data.preimages[0].blob;
        let program = parser::program::preimage(blob, &[]).unwrap();
        println!("program: {:?}", program.code.len());
    }
}
