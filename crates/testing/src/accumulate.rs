//! Accumulate tests

use anyhow::Result;
use runtime::tx;
use score::{
    OpaqueHash, TimeSlot,
    service::{WorkReport, WorkReportJson},
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
pub use types::*;

include!(concat!(env!("OUT_DIR"), "/accumulate.rs"));

/// Run the accumulate test
pub async fn run(test: &specjam::Test) -> Result<()> {
    let input = TestInput::from_json(&test.input)?;
    let output = TestOutput::from_json(&test.output)?;
    let accounts = input.pre_state.accounts();

    // run the accumulate function
    let use_compiler = std::env::var("SPACEVM").is_ok_and(|v| v == "true");
    let accumulation = if use_compiler {
        tx::guarantee::accumulate::<spacevm::SpaceVM, _>(
            input.input.slot,
            input.pre_state.slot,
            input.input.reports,
            &input.pre_state.ready_queue,
            &input.pre_state.accumulated,
            &input.pre_state.privileges.into(),
            &Default::default(),
            &Default::default(),
            accounts.clone(),
            Default::default(),
        )?
    } else {
        tx::guarantee::accumulate::<spacevm::Interpreter, _>(
            input.input.slot,
            input.pre_state.slot,
            input.input.reports,
            &input.pre_state.ready_queue,
            &input.pre_state.accumulated,
            &input.pre_state.privileges.into(),
            &Default::default(),
            &Default::default(),
            accounts.clone(),
            Default::default(),
        )?
    };

    // convert the accounts to the service items
    let mut accounts = self::to_accounts(&accumulation);
    assert_eq!(
        accumulation.records,
        output.post_state.statistics(),
        "statistics mismatch"
    );
    assert_eq!(accumulation.root, output.output.unwrap());
    assert_eq!(
        accumulation.accumulated_queue,
        output.post_state.accumulated
    );
    assert_eq!(accumulation.ready_queue, output.post_state.ready_queue);
    let paccounts = output.post_state.haccounts();

    /* assert_eq!(
        accounts.iter().map(|a| a.id).collect::<Vec<_>>(),
        paccounts.iter().map(|a| a.id).collect::<Vec<_>>(),
        "account length mismatch"
    ); */
    accounts.retain(|a| paccounts.iter().any(|p| p.id == a.id));
    for i in 0..accounts.len() {
        let left = &accounts[i];
        let right = &paccounts[i];
        assert_eq!(left.id, right.id);
        assert_eq!(
            left.data.service, right.data.service,
            "service id ={}",
            left.id
        );
        assert_eq!(left.data.storage, right.data.storage);
        assert_eq!(left.data.preimage_requests, right.data.preimage_requests);
        assert_eq!(left.data.preimages, right.data.preimages);
    }
    assert_eq!(accounts, output.post_state.haccounts());
    assert_eq!(accumulation.privileges, output.post_state.privileges.into());
    Ok(())
}

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

mod types {
    use crate::reports::{ServiceItem, ServiceItemJson};
    use ::account::{Account, Accounts};
    use runtime::Accumulation;
    use score::{
        Entropy, Gas, ServiceId, TimeSlot,
        service::{AccumulatedQueue, Privileges, ReadyQueue, ReadyReportJson, ServiceAccount},
        state::account,
        statistic::{ServiceActivityRecord, ServiceActivityRecordJson},
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;
    use std::collections::BTreeMap;

    /// Convert the accumulation to the accounts
    pub fn to_accounts<R: Accounts>(accumulation: &Accumulation<R>) -> Vec<ServiceItem> {
        let mut items = Vec::new();
        let accounts = accumulation.accounts.accounts();
        for (id, account) in accounts.iter() {
            let account = account.account();
            items.push(ServiceItem {
                id: *id,
                data: (&account).into(),
            });
        }
        items
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

        /// The statistics
        #[json(nested)]
        pub statistics: Vec<RecordWrap>,

        /// The accounts
        #[json(nested)]
        pub accounts: Vec<ServiceItem>,
    }

    impl State {
        /// Get the accounts
        pub fn accounts(&self) -> BTreeMap<u32, ServiceAccount> {
            self.haccounts()
                .iter()
                .map(|item| (item.id, item.clone().into()))
                .collect()
        }

        /// Get the accounts with hashed storage keys
        pub fn haccounts(&self) -> Vec<ServiceItem> {
            let mut services = self.accounts.clone();
            services.iter_mut().for_each(|item| {
                let index = item.id;
                item.data.storage.iter_mut().for_each(|storage| {
                    storage.key = account::storage(index, &storage.key).to_vec();
                });
            });

            services
        }

        /// Get the statistics
        pub fn statistics(&self) -> BTreeMap<ServiceId, ServiceActivityRecord> {
            self.statistics
                .iter()
                .map(|item| (item.id, item.record.clone()))
                .collect()
        }
    }

    /// Record wrapper
    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct RecordWrap {
        /// The service id
        pub id: ServiceId,

        /// The record
        #[json(nested)]
        pub record: ServiceActivityRecord,
    }

    /// Privileges wrapper
    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct PrivilegesWrap {
        /// The bless service id
        pub bless: ServiceId,

        /// The assign service id
        #[json(Vec<ServiceId>)]
        pub assign: score::CoreAssignments,

        /// The designate service id
        pub designate: ServiceId,

        /// The registrar service id
        pub register: ServiceId,

        /// The always accumulate service ids
        #[json(nested)]
        pub always_acc: Vec<AlwaysAccumulateMapItem>,
    }

    impl From<PrivilegesWrap> for Privileges {
        fn from(value: PrivilegesWrap) -> Self {
            Privileges {
                bless: value.bless,
                designate: value.designate,
                register: value.register,
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
                register: value.register,
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
}
