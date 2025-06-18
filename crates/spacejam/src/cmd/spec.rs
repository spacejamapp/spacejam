//! Spec related commands

use crate::chain;
use clap::Parser;
use runtime::{
    storage::{Commit, KVStorage, MemoryDb},
    Storage,
};
use score::{
    service::{GasLimit, ServiceAccount, ServiceData},
    state::{ServiceField, StateKey, StateKeyInfo, StateKeyLike},
};
use std::{collections::BTreeMap, path::PathBuf};

/// Spec related utils
#[derive(Parser)]
pub enum Spec {
    /// Inspect the storage of the node spec
    Inspect,
}

impl Spec {
    /// Run the command
    pub fn run(self, path: PathBuf) -> anyhow::Result<()> {
        match self {
            Self::Inspect => Self::inspect(path),
        }
    }

    fn inspect(path: PathBuf) -> anyhow::Result<()> {
        let spec = std::fs::read_to_string(path)?;
        let spec = serde_json::from_str::<chain::Spec>(&spec)?;
        let memdb = MemoryDb::default();
        let mut accounts = BTreeMap::new();
        let mut commit = Commit::default();

        for (key, value) in spec.genesis_state.into_iter() {
            let key = hex::decode(key.trim_start_matches("0x"))?;
            let value = hex::decode(value.trim_start_matches("0x"))?;
            let info = key.as_state_key().info();
            commit.set(
                key.clone().try_into().map_err(|_| {
                    anyhow::anyhow!(
                        "invalid key: 0x{}, expected 31 bytes, got {}",
                        hex::encode(&key),
                        key.len()
                    )
                })?,
                value.clone(),
            );

            match info {
                StateKey::Account {
                    service,
                    field: ServiceField::Data,
                } => {
                    let account: ServiceData = codec::decode(&value)?;
                    accounts.entry(service).or_insert_with(|| ServiceAccount {
                        balance: account.balance,
                        code: account.code,
                        gas: GasLimit {
                            transfer: account.transfer,
                            accumulate: account.accumulate,
                        },
                        ..Default::default()
                    });
                }

                _ => {}
            }
        }
        memdb.commit(commit)?;
        let mut state = memdb.state()?;
        state.accounts = accounts;
        println!("{:?}", state);
        Ok(())
    }
}
