//! Spec related commands

use crate::chain;
use clap::Parser;
use runtime::storage::{Column, Commit, KVStorage, MemoryDb, StateStorage};
use score::{
    service::{ServiceAccount, ServiceData},
    state::{ServiceField, StateKey, StateKeyInfo, StateKeyLike},
    Account,
};
use spacejson::Json;
use std::{collections::BTreeMap, path::PathBuf};

/// Spec related utils
#[derive(Parser)]
pub enum State {
    /// Inspect the storage of the node spec
    Inspect { spec: PathBuf },
}

impl State {
    /// Run the command
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Inspect { spec } => Self::inspect(spec),
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
            let mut skey = [0; 31];
            skey.copy_from_slice(&key[..31]);
            commit.set(skey, value.clone());

            match info {
                StateKey::Account {
                    service,
                    field: ServiceField::Data,
                } => {
                    let account: ServiceData = codec::decode(&value)?;
                    let entry = accounts
                        .entry(service)
                        .or_insert_with(ServiceAccount::default);
                    entry.balance = account.balance;
                    entry.code = account.code;
                    entry.transfer_gas = account.transfer;
                    entry.accumulate_gas = account.accumulate;
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Preimage,
                } => {
                    accounts
                        .entry(service)
                        .or_insert_with(ServiceAccount::default)
                        .insert_preimage(crypto::blake2b(&value), value);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Storage,
                } => {
                    accounts
                        .entry(service)
                        .or_insert_with(ServiceAccount::default)
                        .storage
                        .insert(key, value);
                }
                StateKey::Account {
                    service,
                    field: ServiceField::Lookup { length },
                } => {
                    let value = codec::decode(&value)?;
                    let mut skey = [0; 32];
                    skey[9..].copy_from_slice(&key[8..31]);
                    accounts
                        .entry(service)
                        .or_insert_with(ServiceAccount::default)
                        .lookup
                        .insert((skey, length), value);
                }

                _ => {}
            }
        }

        memdb.commit(Column::State, commit)?;
        let mut state = memdb.state()?;
        state.accounts = accounts;
        println!("{:?}", state.to_json());
        Ok(())
    }
}
