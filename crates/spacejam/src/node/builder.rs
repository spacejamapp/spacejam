//! Configuration for the spacejam node

use crate::{node::Genesis, Context, Spacejam};
use network::{Context as _, Network};
use score::{
    block::BlockInfo,
    state::{key::CURRENT_VALIDATORS, Storage},
    validator::{Validator, ValidatorData},
    Block,
};
use spacejson::Json;
use std::{fs, path::PathBuf};
use tokio::sync::mpsc;

/// Spacejam node builder
#[derive(Clone, Default)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Builder {
    /// The seed of the validator
    #[cfg_attr(feature = "cmd", arg(long))]
    validator: String,

    /// The database path
    #[cfg_attr(feature = "cmd", arg(long, default_value = "spacejam.db"))]
    db: PathBuf,

    /// The genesis path
    #[cfg_attr(feature = "cmd", arg(long, default_value = "genesis.json"))]
    genesis: PathBuf,

    /// If force the node authoring blocks
    #[cfg_attr(feature = "cmd", arg(long, default_value = "false"))]
    authoring: bool,

    /// The network configuration
    #[cfg_attr(feature = "cmd", command(flatten))]
    network: network::Config,
}

impl Builder {
    /// Build the node
    pub async fn build<S: Storage + 'static, V: Validator + TryFrom<String> + 'static>(
        self,
    ) -> anyhow::Result<Spacejam<S, V>> {
        let (tx, rx) = mpsc::channel(100);
        let validator = V::try_from(self.validator.clone())
            .map_err(|_| anyhow::anyhow!("Invalid seed {:?}", self.validator))?;
        let context = Context::new(validator, S::open(self.db.clone())?, tx);

        // Initialize the database
        if context.db.is_empty() {
            let genesis = fs::read_to_string(self.genesis.clone())
                .map_err(|_| anyhow::anyhow!("Failed to read genesis file {:?}", self.genesis))?;
            let genesis: Genesis = serde_json::from_str(&genesis)
                .map_err(|_| anyhow::anyhow!("Failed to parse genesis file {:?}", self.genesis))?;

            // insert the genesis block into database
            let block: Block = genesis.block.try_into()?;
            let recent: Vec<BlockInfo> = vec![block.header.into()];
            context
                .db
                .set(score::state::key::RECENT_BLOCKS, codec::encode(&recent)?)?;

            // set up initial validators
            let validators = genesis
                .validators
                .into_iter()
                .map(Json::from_json)
                .collect::<anyhow::Result<Vec<ValidatorData>>>()?;
            let encoded = codec::encode(&validators)?;
            context.db.set(CURRENT_VALIDATORS, encoded)?;
        }

        // Initialize the network
        //
        // TODO: add config to the inner channel
        let network = Network::new(self.network, rx, context.keypair()).await?;
        Ok(Spacejam {
            context,
            network,
            authoring: self.authoring,
        })
    }
}
