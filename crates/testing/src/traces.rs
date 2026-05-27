//! state transition traces

use ::account::{Account, Accounts};
use pvm::Pvm;
use runtime::{
    storage::{Column, KVStorage, MemoryDb, StateStorage},
    tx::{self, block::header, ticket::lazy},
};
use score::{
    EntropyBuffer, OpaqueHash, TrieKey,
    block::{Block, BlockInfo, BlockJson, Header, History, Mmr},
    safrole::{Safrole, ValidatorIter, ValidatorsData},
    service::{AccumulatedQueue, Privileges, ReadyQueue, ServiceInfo},
    state::{StateKeyInfo, StateKeyLike, account, key},
    statistic::Statistics,
    vm::CommitmentMap,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::{collections::HashMap, sync::Arc, time::Instant};

mod fallback {
    include!(concat!(env!("OUT_DIR"), "/traces_fallback.rs"));
}

mod fuzzy {
    include!(concat!(env!("OUT_DIR"), "/traces_fuzzy.rs"));
}

mod fuzzy_light {
    include!(concat!(env!("OUT_DIR"), "/traces_fuzzy_light.rs"));
}

mod safrole {
    include!(concat!(env!("OUT_DIR"), "/traces_safrole.rs"));
}

mod preimages {
    include!(concat!(env!("OUT_DIR"), "/traces_preimages.rs"));
}

mod preimages_light {
    include!(concat!(env!("OUT_DIR"), "/traces_preimages_light.rs"));
}

mod storage {
    include!(concat!(env!("OUT_DIR"), "/traces_storage.rs"));
}

mod storage_light {
    include!(concat!(env!("OUT_DIR"), "/traces_storage_light.rs"));
}

pub async fn run(test: &specjam::Test) -> anyhow::Result<bool> {
    if let Some(s) = test.input.as_json()
        && s.len() == 31
    {
        return Ok(false);
    }
    let memdb = Arc::new(MemoryDb::default());
    let (input, output) = decode(test)?;
    for keyval in input.pre_state.keyvals.clone() {
        memdb
            .state_set(keyval.key, keyval.value)
            .expect("failed to set keyval");
    }

    let use_compiler = std::env::var("SPACEVM").is_ok_and(|v| v == "true");
    if use_compiler {
        self::run_single::<spacevm::SpaceVM, _>(memdb, input, output).await
    } else {
        self::run_single::<spacevm::Interpreter, _>(memdb, input, output).await
    }
}

/// Run the traces test
pub async fn run_single<Vm: Pvm, S: runtime::Storage>(
    memdb: Arc<S>,
    input: TestInput,
    output: TestOutput,
) -> anyhow::Result<bool> {
    let block: Block = input.block;
    let mut pkeys = Vec::new();
    let is_ok = tx::block::process::<Vm>(block, memdb.clone())
        .inspect_err(|e| tracing::error!("{e:?}"))
        .is_ok();

    for KeyValue { key, value } in output.post_state.keyvals {
        let info = key.as_state_key().info();
        let encoded = hex::encode(&key);
        let Some(result) = memdb.state_get(&key)? else {
            tracing::error!(
                "{info:?} key=0x{encoded} value=0x{} not exists in spacejam",
                hex::encode(&value)
            );
            continue;
        };

        pkeys.push(key.clone());
        if value != result {
            tracing::error!("keyval mismatch: {info:?}: 0x{encoded}");
            tracing::error!(
                "\npolkajam={}\nspacejam={}",
                hex::encode(&value),
                hex::encode(&result)
            );
        } else {
            tracing::trace!("keyval matched: {info:?}: 0x{encoded}");
        }

        if key == key::ACCUMULATION_LOGS && value != result {
            let polkajam: CommitmentMap = codec::decode(&value)?;
            let spacejam: CommitmentMap = codec::decode(&result)?;
            tracing::debug!(
                "polkajam: {:?}",
                polkajam
                    .iter()
                    .map(|(k, v)| (k, hex::encode(v)))
                    .collect::<Vec<_>>()
            );
            tracing::debug!(
                "spacejam: {:?}",
                spacejam
                    .iter()
                    .map(|(k, v)| (k, hex::encode(v)))
                    .collect::<Vec<_>>()
            );
        }

        if key == key::TIMESLOT && value != result {
            let polkajam: u32 = codec::decode(&value)?;
            let timeslot: u32 = codec::decode(&result)?;
            tracing::debug!("polkajam: {:?}", polkajam);
            tracing::debug!("spacejam: {:?}", timeslot);
        }

        if key == key::STATISTICS && value != result {
            let polkajam: Statistics = codec::decode(&value)?;
            let statistics: Statistics = codec::decode(&result)?;
            tracing::debug!("polkajam: {:#?}", polkajam.to_json());
            tracing::debug!("spacejam: {:#?}", statistics.to_json());
        }

        if key == key::RECENT_BLOCKS && value != result {
            let polkajam: History = codec::decode(&value)?;
            let recent: History = codec::decode(&result)?;
            tracing::debug!("polkajam: {:?}", polkajam.to_json());
            tracing::debug!("spacejam: {:?}", recent.to_json());
        }

        if key == key::PRIVILEGED_SERVICE && value != result {
            let polkajam: Privileges = codec::decode(&value)?;
            let spacejam: Privileges = codec::decode(&result)?;
            tracing::debug!("polkajam: {:?}", polkajam);
            tracing::debug!("spacejam: {:?}", spacejam);
        }

        if key == key::SAFROLE && value != result {
            let polkajam: Safrole = codec::decode(&value)?;
            let spacejam: Safrole = codec::decode(&result)?;
            tracing::debug!("polkajam: {:?}", polkajam.to_json());
            tracing::debug!("spacejam: {:?}", spacejam.to_json());
        }

        if key == key::DRAWN_VALIDATORS && value != result {
            let polkajam: ValidatorsData = codec::decode(&value)?;
            let spacejam: ValidatorsData = codec::decode(&result)?;
            tracing::debug!(
                "polkajam-ed25519: {:?}",
                polkajam
                    .iter()
                    .map(|v| hex::encode(v.ed25519))
                    .collect::<Vec<_>>()
            );
            tracing::debug!(
                "spacejam-ed25519: {:?}",
                spacejam
                    .iter()
                    .map(|v| hex::encode(v.ed25519))
                    .collect::<Vec<_>>()
            );
        }

        if key.starts_with(&[255]) && value != result {
            let polkajam: ServiceInfo = codec::decode(&value)?;
            let spacejam: ServiceInfo = codec::decode(&result)?;
            tracing::debug!("polkajam: {:#?}", polkajam.to_json());
            tracing::debug!("spacejam: {:#?}", spacejam.to_json());
        }
    }

    // check if spacejam left extra keyvals
    for pair in memdb.state_iter()? {
        let (key, value) = pair?;
        if pkeys.contains(&key) {
            continue;
        }

        let info = key.as_state_key().info();
        tracing::error!(
            "extra keyval: {info:?} key=0x{} value=0x{}...",
            hex::encode(&key),
            hex::encode(&value[..std::cmp::min(32, value.len())])
        );
    }

    let state_root = memdb.root().expect("failed to get state root");
    assert_eq!(state_root, output.post_state.state_root);
    Ok(is_ok)
}

/// State transition trace input
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestInput {
    /// The state
    #[json(nested)]
    pub pre_state: State,

    /// The block
    #[json(nested)]
    pub block: Block,
}

/// State transition trace output
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestOutput {
    /// The post-state
    #[json(nested)]
    pub post_state: State,
}

/// State transition trace state
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct State {
    /// The state root
    #[json(hex)]
    pub state_root: OpaqueHash,

    /// The key-values
    #[json(nested)]
    pub keyvals: Vec<KeyValue>,
}

impl State {
    /// Get the key-values
    pub fn keyvals(&self) -> HashMap<Vec<u8>, Vec<u8>> {
        self.keyvals
            .clone()
            .into_iter()
            .map(|kv| (kv.key, kv.value))
            .collect::<HashMap<_, _>>()
    }
}

/// State transition trace key-value
#[derive(Debug, Serialize, Deserialize, Json, Clone)]
pub struct KeyValue {
    /// The key
    #[json(hex)]
    pub key: Vec<u8>,

    /// The value
    #[json(hex)]
    pub value: Vec<u8>,
}

/// Decode a `Test` into its input/output.
pub fn decode(test: &specjam::Test) -> anyhow::Result<(TestInput, TestOutput)> {
    if let Some(bytes) = test.input.as_bin() {
        return from_bin(bytes);
    }
    let input = test
        .input
        .as_json()
        .ok_or_else(|| anyhow::anyhow!("trace input: expected JSON or bin"))?;
    let output = test
        .output
        .as_json()
        .ok_or_else(|| anyhow::anyhow!("trace output: expected JSON"))?;
    Ok((TestInput::from_json(input)?, TestOutput::from_json(output)?))
}

/// Decode a binary trace into test input/output.
pub fn from_bin(bytes: &[u8]) -> anyhow::Result<(TestInput, TestOutput)> {
    let bin: bin::BinTrace = codec::decode(bytes)?;
    Ok((
        TestInput {
            pre_state: bin.pre_state.into_state(),
            block: bin.block,
        },
        TestOutput {
            post_state: bin.post_state.into_state(),
        },
    ))
}

mod bin {
    use super::*;

    /// A binary trace
    #[derive(Deserialize)]
    pub struct BinTrace {
        pub pre_state: BinState,
        pub block: Block,
        pub post_state: BinState,
    }

    /// A binary state
    #[derive(Deserialize)]
    pub struct BinState {
        pub state_root: OpaqueHash,
        pub keyvals: Vec<BinKeyValue>,
    }

    /// A binary key-value
    #[derive(Deserialize)]
    pub struct BinKeyValue {
        pub key: TrieKey,
        pub value: Vec<u8>,
    }

    /// Convert a binary state into a state
    impl BinState {
        pub fn into_state(self) -> State {
            State {
                state_root: self.state_root,
                keyvals: self
                    .keyvals
                    .into_iter()
                    .map(|kv| KeyValue {
                        key: kv.key.to_vec(),
                        value: kv.value,
                    })
                    .collect(),
            }
        }
    }
}
