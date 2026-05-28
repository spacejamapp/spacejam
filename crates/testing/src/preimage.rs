//! Preimage tests

use account::{Account, Accounts};
use runtime::tx;
use score::service::ServiceAccount;
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;
use types::*;

include!(concat!(env!("OUT_DIR"), "/preimages.rs"));

pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    // The preimages STF `Output` is `CHOICE { ok NULL, err ErrorCode }`; the
    // test recomputes it below, so the decoded value is discarded.
    let (preimages, pre, _output, post) =
        codec::decode::<(Input, RawState, Result<(), u8>, RawState)>(test.input.expect_bin()?)?;
    let input = TestInput { input: preimages, pre_state: pre.into() };
    let output = TestOutput { post_state: post.into() };

    // Validate post state
    let mut accounts = to_accounts(input.pre_state.accounts.clone());
    if tx::preimage::validate(&mut accounts, &input.input.preimages).is_err() {
        assert_eq!(input.pre_state, output.post_state);
        return Ok(());
    }
    let accounts =
        tx::preimage::accounts(input.input.slot, input.input.preimages.clone(), accounts);
    assert_eq!(
        accounts
            .accounts()
            .iter()
            .map(|(id, account)| (*id, account.account()))
            .collect::<BTreeMap<_, _>>(),
        self::to_accounts(output.post_state.accounts)
    );

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
        accounts.insert(acc.id, acc.data.into());
    }
    accounts
}

// TODO: clean types later
mod types {
    use score::{
        OpaqueHash,
        extrinsic::{Preimage, PreimageJson},
        service::ServiceAccount,
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
                account
                    .lookup
                    .insert((lookup.key.hash, lookup.key.length), lookup.value);
            }

            account
        }
    }

    #[derive(Debug, Serialize, Deserialize, Json, Clone, PartialEq, Eq)]
    pub struct TState {
        #[json(nested)]
        pub accounts: Vec<Account>,
    }

    /// The preimages STF `State` raw layout: `(accounts, services-statistics)`.
    /// The statistics records aren't asserted on, so they're discarded.
    pub type RawState = (
        Vec<Account>,
        Vec<(score::ServiceId, score::statistic::ServiceActivityRecord)>,
    );

    impl From<RawState> for TState {
        fn from((accounts, _stats): RawState) -> Self {
            TState { accounts }
        }
    }
}
