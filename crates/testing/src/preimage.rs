//! Preimage tests

use runtime::tx;
use score::{service::ServiceAccount, Account, Accounts};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;
use types::*;

// FIXME: skipping the preimage tests since it's currently outdated.
//
// include!(concat!(env!("OUT_DIR"), "/preimages.rs"));

pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let input = TestInput::from_json(&test.input)?;
    let output = TestOutput::from_json(&test.output)?;

    // Validate post state
    let accounts = to_accounts(input.pre_state.accounts.clone());
    let result = tx::preimage::accounts(input.input.slot, &input.input.preimages, accounts);
    if let Ok(accounts) = result {
        assert_eq!(
            accounts
                .accounts()
                .iter()
                .map(|(id, account)| (*id, account.account()))
                .collect::<BTreeMap<_, _>>(),
            self::to_accounts(output.post_state.accounts)
        );
    } else {
        assert_eq!(input.pre_state, output.post_state);
    }

    Ok(())
}

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
        accounts.insert(acc.id, acc.data.into_service_account(acc.id));
    }
    accounts
}

// TODO: clean types later
mod types {
    use score::{
        extrinsic::{Preimage, PreimageJson},
        service::ServiceAccount,
        AccountInnerKey, OpaqueHash,
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

    impl AccountInfo {
        pub fn into_service_account(self, index: u32) -> ServiceAccount {
            let mut account = ServiceAccount::default();
            for preimage in self.preimages {
                let ikey = AccountInnerKey::Preimage(index, preimage.hash);
                account.preimage.insert(ikey, preimage.blob);
            }

            for lookup in self.lookup_meta {
                let mut slots = [0; 3];
                slots[..lookup.value.len()].copy_from_slice(&lookup.value);
                let ikey = AccountInnerKey::Lookup(index, lookup.key.hash, lookup.key.length);
                account.lookup.insert(ikey, slots.to_vec());
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
