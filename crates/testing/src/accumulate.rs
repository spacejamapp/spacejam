//! Accumulate tests

use anyhow::Result;
use runtime::tx;
use score::{
    service::{WorkReport, WorkReportJson},
    OpaqueHash, TimeSlot,
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
    let mut accumulation = if use_compiler {
        tx::guarantee::accumulate::<jastime::Compiler, _>(
            input.input.slot,
            input.pre_state.slot,
            input.input.reports,
            &input.pre_state.ready_queue,
            &input.pre_state.accumulated,
            &input.pre_state.privileges.into(),
            &Default::default(),
            accounts.clone(),
            Default::default(),
        )
        .await?
    } else {
        tx::guarantee::accumulate::<jastime::Interpreter, _>(
            input.input.slot,
            input.pre_state.slot,
            input.input.reports,
            &input.pre_state.ready_queue,
            &input.pre_state.accumulated,
            &input.pre_state.privileges.into(),
            &Default::default(),
            accounts.clone(),
            Default::default(),
        )
        .await?
    };
    accumulation.root = Default::default();

    // convert the accounts to the service items
    let accounts = self::to_accounts(&accumulation);
    // assert_eq!(accumulation.records, output.post_state.statistics());
    assert_eq!(accumulation.root, output.output.unwrap());
    assert_eq!(
        accumulation.accumulated_queue,
        output.post_state.accumulated
    );
    assert_eq!(accumulation.ready_queue, output.post_state.ready_queue);
    for (idx, account) in accounts.iter().enumerate() {
        assert_eq!(
            account.data.service.total,
            output.post_state.accounts[idx].data.service.total
        );
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
    use score::{
        service::{AccumulatedQueue, Privileges, ReadyQueue, ReadyReportJson, ServiceAccount},
        state::account,
        statistic::{ServiceActivityRecord, ServiceActivityRecordJson},
        vm::Accumulation,
        Account, Accounts, Entropy, Gas, ServiceId, TimeSlot,
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
            if account.preimage.contains_key(&account.info.code) {
                items.push(ServiceItem {
                    id: *id,
                    data: (&account).into(),
                });

                continue;
            }

            for other in accounts.values() {
                let other = other.account();
                if other.info.code != account.info.code
                    || !other.preimage.contains_key(&account.info.code)
                {
                    continue;
                }

                let mut account = account.clone();
                let blob = other
                    .preimage
                    .get(&account.info.code)
                    .cloned()
                    .unwrap_or_default();
                account
                    .lookup
                    .insert((account.info.code, blob.len() as u32), Default::default());
                account.preimage.insert(account.info.code, blob);

                let mut item: ServiceItem = ServiceItem {
                    id: *id,
                    data: (&account).into(),
                };

                item.data.preimages.retain(|k| k.hash != account.info.code);
                items.push(item);
            }
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

        /// The designate service id
        pub designate: ServiceId,

        /// The assign service id
        #[json(Vec<ServiceId>)]
        pub assign: [ServiceId; score::CORES_COUNT],

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
}
