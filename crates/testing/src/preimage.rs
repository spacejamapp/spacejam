//! Preimage tests

use score::service::ServiceAccount;
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;
use types::*;

/// Test input.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: TState,
}

/// Test output.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestOutput {
    #[json(nested)]
    pub post_state: TState,
}

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Test {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: TState,
    #[json(nested)]
    pub post_state: TState,
}

/// Convert test input to state.
pub fn to_accounts(accs: Vec<types::Account>) -> BTreeMap<u32, ServiceAccount> {
    let mut accounts = BTreeMap::new();
    for acc in accs {
        accounts.insert(acc.id, acc.data.into());
    }
    accounts
}

/* include!(concat!(env!("OUT_DIR"), "/preimages.rs")); */

// TODO: clean types later
mod types {
    use score::{
        extrinsic::{Preimage, PreimageJson},
        service::ServiceAccount,
        OpaqueHash,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    #[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct Input {
        #[json(nested)]
        pub preimages: Vec<Preimage>,
        pub slot: u32,
    }

    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct Account {
        /// Account ID
        pub id: u32,

        /// Account info
        #[json(nested)]
        pub data: AccountInfo,
    }

    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct TPreimage {
        #[json(hex)]
        pub hash: OpaqueHash,
        #[json(hex)]
        pub blob: Vec<u8>,
    }

    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct HistoryKey {
        #[json(hex)]
        pub hash: OpaqueHash,
        pub length: u32,
    }

    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct History {
        #[json(nested)]
        pub key: HistoryKey,
        pub value: Vec<u32>,
    }

    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct AccountInfo {
        #[json(nested)]
        pub preimages: Vec<TPreimage>,
        #[json(nested)]
        pub lookup_meta: Vec<History>,
    }

    impl From<AccountInfo> for ServiceAccount {
        fn from(info: AccountInfo) -> Self {
            let mut account = ServiceAccount::default();
            for preimage in info.preimages {
                account.preimage.insert(preimage.hash, preimage.blob);
            }

            for lookup in info.lookup_meta {
                let mut slots = [0; 3];
                slots[..lookup.value.len()].copy_from_slice(&lookup.value);
                account
                    .lookup
                    .insert((lookup.key.hash, lookup.key.length), slots.to_vec());
            }

            account
        }
    }

    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct TState {
        #[json(nested)]
        pub accounts: Vec<Account>,
    }
}
