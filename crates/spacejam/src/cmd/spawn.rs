//! Spawn the node

use crate::Context;
use clap::Parser;
use network::Network;
use score::{
    genesis::Genesis,
    state::{key::CURRENT_VALIDATORS, Storage},
    validator::{Validator, ValidatorData},
};
use spacejson::Json;
use std::{fs, net::SocketAddr, path::PathBuf};

/// Spawn the node
#[derive(Parser)]
pub struct Spawn {
    /// Path to the database
    #[arg(short, long, default_value = "chain.db")]
    pub db: PathBuf,

    /// Path to the genesis file
    #[arg(short, long, default_value = "genesis.json")]
    pub genesis: PathBuf,

    /// Metrics address
    #[arg(short, long, default_value = "0.0.0.0:0")]
    pub metrics: SocketAddr,

    /// Validator secret phrase, accepts a hex string or a number
    ///
    /// TODO: if no validator is provided, the node will not author blocks
    #[arg(long)]
    pub validator: String,

    /// If force this node authoring blocks
    #[arg(short, long, default_value = "false")]
    pub author: bool,

    /// The network configuration
    #[command(flatten)]
    pub network: network::Config,
}

impl Spawn {
    /// Run the command
    pub async fn run<S: Storage + 'static, V: Validator + TryFrom<String> + 'static>(
        &self,
    ) -> anyhow::Result<()> {
        // Parse the validator secret
        let validator =
            V::try_from(self.validator.clone()).map_err(|_| anyhow::anyhow!("Invalid seed"))?;
        let spacejam = Context::new(validator, S::open(self.db.clone())?);

        // Initialize the database
        if spacejam.db.is_empty() {
            let genesis = fs::read_to_string(self.genesis.clone())
                .map_err(|_| anyhow::anyhow!("Failed to read genesis file {:?}", self.genesis))?;
            let genesis: Genesis = serde_json::from_str(&genesis)
                .map_err(|_| anyhow::anyhow!("Failed to parse genesis file {:?}", self.genesis))?;
            let validators = genesis
                .validators
                .into_iter()
                .map(Json::from_json)
                .collect::<anyhow::Result<Vec<ValidatorData>>>()?;
            let encoded = codec::encode(&validators)?;
            spacejam.db.set(CURRENT_VALIDATORS, encoded)?;
        }

        // Initialize the network
        //
        // TODO: initialize the network with the given config from input
        let mut network = Network::new(self.network.clone(), Box::new(spacejam)).await?;
        let metrics = network.metrics.clone();
        tokio::select! {
            _ = crate::metrics::serve(self.metrics, metrics) => {}
            _ = network.spawn() => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        Ok(())
    }
}
