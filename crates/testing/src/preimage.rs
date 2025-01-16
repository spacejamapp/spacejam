use score::State;
use serde::{Deserialize, Serialize};
use spacejson::Json;
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
pub fn to_state(accs: Vec<types::Account>) -> State {
    let mut state = State::default();
    for acc in accs {
        state.accounts.insert(acc.id, acc.info.into());
    }
    state
}

crate::impl_tests! {
    preimages,
    preimage_needed_1,
    preimage_needed_2,
    preimage_not_needed_1,
    preimage_not_needed_2
}

// TODO: clean types later
mod types {
    use score::{
        extrinsic::{Preimage, PreimageJson},
        service::ServiceAccount,
        OpaqueHash,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct Input {
        #[json(nested)]
        pub preimages: Vec<Preimage>,
        pub slot: u32,
    }

    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct Account {
        /// Account ID
        pub id: u32,

        /// Account info
        #[json(nested)]
        pub info: AccountInfo,
    }

    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct TPreimage {
        #[json(hex)]
        pub hash: OpaqueHash,
        #[json(hex)]
        pub blob: Vec<u8>,
    }

    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct HistoryKey {
        #[json(hex)]
        pub hash: OpaqueHash,
        pub length: u32,
    }

    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct History {
        #[json(nested)]
        pub key: HistoryKey,
        pub value: Vec<u32>,
    }

    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct AccountInfo {
        #[json(nested)]
        pub preimages: Vec<TPreimage>,
        #[json(nested)]
        pub history: Vec<History>,
    }

    impl From<AccountInfo> for ServiceAccount {
        fn from(info: AccountInfo) -> Self {
            let mut account = ServiceAccount::default();
            for preimage in info.preimages {
                account.preimage.insert(preimage.hash, preimage.blob);
            }

            for lookup in info.history {
                let mut slots = [0; 3];
                slots[..lookup.value.len()].copy_from_slice(&lookup.value);
                account
                    .lookup
                    .insert((lookup.key.hash, lookup.key.length), slots);
            }

            account
        }
    }

    #[derive(Debug, Serialize, Deserialize, Json)]
    pub struct TState {
        #[json(nested)]
        pub accounts: Vec<Account>,
    }
}
