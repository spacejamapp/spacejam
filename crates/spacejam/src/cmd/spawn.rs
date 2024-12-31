//! Spawn the node

use crate::{Config, SpaceJam};
use clap::Parser;
use score::{
    config::Genesis,
    state::{
        key::{CURRENT_VALIDATORS, TIMESLOT},
        Storage,
    },
    validator::ValidatorData,
};
use spacejson::Json;
use std::{fs, path::PathBuf};

/// Spawn the node
#[derive(Default, Parser)]
pub struct Spawn {
    /// Path to the database
    #[arg(short, long, default_value = "chain.db")]
    pub db: PathBuf,

    /// Path to the genesis file
    #[arg(short, long, default_value = "genesis.json")]
    pub genesis: PathBuf,
}

impl Spawn {
    /// Run the command
    pub fn run<C: Config>(&self) -> anyhow::Result<()> {
        let mut spacejam: SpaceJam<C> =
            SpaceJam::new(C::Db::open(self.db.clone())?, C::Validator::default());

        if spacejam.db.is_empty() {
            let genesis = fs::read_to_string(self.genesis.clone())?;
            let genesis: Genesis = serde_json::from_str(&genesis)?;
            let validators = genesis
                .validators
                .into_iter()
                .map(Json::from_json)
                .collect::<anyhow::Result<Vec<ValidatorData>>>()?;
            let encoded = codec::encode(&validators)?;

            spacejam.db.set(CURRENT_VALIDATORS, encoded)?;
        }

        // TODO: confirm slot vs block.
        let slot = spacejam.db.get(TIMESLOT)?.unwrap_or(vec![]);
        let mut slot: u32 = codec::decode(&slot).unwrap_or(0);

        loop {
            let block = spacejam.mine()?;
            slot += 1;
            tracing::info!("mined block #{}: 0x{}", slot, hex::encode(block.hash()?));
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}
