use paste::paste;
use score::{
    extrinsic::{Preimage, PreimageJson},
    service::ServiceAccount,
    OpaqueHash, State,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Input {
    #[json(nested)]
    preimages: Vec<Preimage>,
    slot: u32,
}

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Account {
    /// Account ID
    id: u32,

    /// Account info
    #[json(nested)]
    info: AccountInfo,
}

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TPreimage {
    #[json(hex)]
    hash: OpaqueHash,
    #[json(hex)]
    blob: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct HistoryKey {
    #[json(hex)]
    pub hash: OpaqueHash,
    length: u32,
}

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct History {
    #[json(nested)]
    key: HistoryKey,
    value: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct AccountInfo {
    #[json(nested)]
    preimages: Vec<TPreimage>,
    #[json(nested)]
    history: Vec<History>,
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
    accounts: Vec<Account>,
}

#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Test {
    #[json(nested)]
    input: Input,
    #[json(nested)]
    pre_state: TState,
    #[json(nested)]
    post_state: TState,
}

fn to_state(accs: Vec<Account>) -> State {
    let mut state = State::default();
    for acc in accs {
        state.service_accounts.insert(acc.id, acc.info.into());
    }
    state
}

impl Test {
    pub fn run(self) -> anyhow::Result<()> {
        let pre: State = to_state(self.pre_state.accounts);
        let post: State = to_state(self.post_state.accounts);

        let result = preimage::handle(pre.clone(), self.input.slot, self.input.preimages)
            .unwrap_or(pre.clone());
        assert_eq!(result, post);
        Ok(())
    }
}

#[allow(unused_macros)]
macro_rules! impl_preimage_tests {
    ($name:ident) => {
        paste! {
            #[test]
            fn [<$name:snake>]() -> anyhow::Result<()> {
                let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                root.extend(["jamtestvectors", "preimages", "data"]);

                let pattern = stringify!($name).split("_").collect::<Vec<&str>>();
                let mut name = pattern[..pattern.len() - 1].join("_");
                name.push_str(&format!(
                    "-{}",
                    pattern.last().expect("pattern must have at least one element")
                ));

                root.push(name);
                root.set_extension("json");

                let json = fs::read_to_string(root)?;
                Test::from_json(&json)?.run()
            }
        }
    };
    ($($name:ident),*) => {
        $(impl_preimage_tests!($name);)*
    };
}

impl_preimage_tests! {
    preimage_needed_1,
    preimage_needed_2,
    preimage_not_needed_1,
    preimage_not_needed_2
}
