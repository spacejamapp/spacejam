//! Spawn the node

use crate::{Config, SpaceJam};
use clap::Parser;
use network::Network;
use score::{
    config::Genesis,
    state::{key::CURRENT_VALIDATORS, Storage},
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
    pub async fn run<C: Config + 'static>(&self) -> anyhow::Result<()> {
        let spacejam: SpaceJam<C> =
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

        let mut network = Network::new(Default::default(), Box::new(spacejam))
            .await
            .expect("failed to create network");
        network.spawn().await;
        Ok(())
    }
}
