//! Accumulate tests

use score::{
    service::{WorkReport, WorkReportJson},
    OpaqueHash, TimeSlot,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
pub use types::*;

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

include!(concat!(env!("OUT_DIR"), "/accumulate.rs"));

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
            if account.preimage.contains_key(&account.code) {
                items.push(ServiceItem {
                    id: *id,
                    data: (&account).into(),
                });

                continue;
            }

            for other in accounts.values() {
                let other = other.account();
                if other.code != account.code || !other.preimage.contains_key(&account.code) {
                    continue;
                }

                let mut account = account.clone();
                let blob = other
                    .preimage
                    .get(&account.code)
                    .cloned()
                    .unwrap_or_default();
                account
                    .lookup
                    .insert((account.code, blob.len() as u32), Default::default());
                account.preimage.insert(account.code, blob);

                let mut item: ServiceItem = ServiceItem {
                    id: *id,
                    data: (&account).into(),
                };

                item.data.preimages.retain(|k| k.hash != account.code);
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
