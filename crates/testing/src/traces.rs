//! state transition traces

use pvmi::Interpreter;
use runtime::{
    storage::{MemoryDb, StateStorage},
    tx,
};
use score::{
    block::{Block, BlockInfo, BlockJson, Header, History, Mmr},
    safrole::ValidatorsData,
    service::{AccumulatedQueue, Privileges, ReadyQueue, ServiceInfo},
    state::{account, key, StateKeyInfo, StateKeyLike},
    statistic::Statistics,
    Account, Accounts, EntropyBuffer, OpaqueHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::sync::Arc;

mod fallback {
    include!(concat!(env!("OUT_DIR"), "/traces_fallback.rs"));
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

mod fuzz {
    include!(concat!(env!("OUT_DIR"), "/traces_local_fuzz.rs"));
    include!(concat!(env!("OUT_DIR"), "/traces_fuzz.rs"));
}

/// Run the traces test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    if test.input.len() == 31 {
        // SKIP the genesis block
        return Ok(());
    }

    let input = TestInput::from_json(&test.input)?;
    let output = TestOutput::from_json(&test.output)?;
    let block: Block = input.block;
    let memdb = Arc::new(MemoryDb::default());

    // 1. verify the state root in pre-stateπ
    let keyvals = input.pre_state.keyvals;
    for keyval in keyvals {
        memdb
            .state_set(keyval.key, keyval.value)
            .expect("failed to set keyval");
    }

    let state_root = memdb.root().expect("failed to get state root");
    assert_eq!(state_root, input.pre_state.state_root);

    // 2. verify the state transition
    let mut pkeys = Vec::new();
    if let Err(e) = tx::transit::<Interpreter>(block, memdb.clone()) {
        tracing::warn!("failed to transit block with error: {e:?}");
    }

    for KeyValue { key, value } in output.post_state.keyvals {
        let info = key.as_state_key().info();
        let encoded = hex::encode(&key);
        let Some(result) = memdb.state_get(&key)? else {
            tracing::error!(
                "{info:?} key=0x{encoded} value=0x{} not exists",
                hex::encode(&value)
            );
            continue;
        };

        pkeys.push(key.clone());
        if value != result {
            tracing::error!(
                "keyval mismatch: {info:?}: 0x{encoded}, expected: 0x{}, got: 0x{}",
                hex::encode(&value),
                hex::encode(&result)
            );
        } else {
            tracing::debug!("keyval matched: {info:?}: 0x{encoded}");
        }

        /* if key == key::STATISTICS && value != result {
            let polkajam: Statistics = codec::decode(&value)?;
            let statistics: Statistics = codec::decode(&result)?;
            tracing::debug!("polkajam: {:#?}", polkajam.to_json());
            tracing::debug!("spacejam: {:#?}", statistics.to_json());
        } */

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

        if key == account::info(3202820706) || key == account::info(0) {
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
    Ok(())
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
