//! Configuration for the spacejam node

use crate::{
    node::{Context, Genesis},
    Spacejam,
};
use network::{Context as _, Network};
use score::{
    block::BlockInfo,
    extrinsic::TicketsOrKeys,
    runtime::{Storage, Validator},
    safrole::{Safrole, ValidatorData},
    state::key,
    Block, EntropyBuffer,
};
use spacejson::Json;
use std::{fs, path::PathBuf, sync::Arc};
use tokio::sync::mpsc;

/// Spacejam node builder
#[derive(Clone, Default)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Builder {
    /// The seed of the validator
    ///
    /// TODO: make this field optional, if not provided, the node will not be a validator.
    #[cfg_attr(feature = "cmd", arg(long))]
    validator: String,

    /// The database path
    #[cfg_attr(feature = "cmd", arg(long, default_value = "spacejam.db"))]
    db: PathBuf,

    /// The genesis path
    #[cfg_attr(feature = "cmd", arg(long, default_value = "genesis.json"))]
    genesis: PathBuf,

    /// The network configuration
    #[cfg_attr(feature = "cmd", command(flatten))]
    network: network::Config,
}

impl Builder {
    /// Build the node
    pub async fn build<
        S: Storage + Send + Sync + 'static + TryFrom<PathBuf, Error = anyhow::Error>,
        V: Validator + Send + Sync + 'static + TryFrom<String> + 'static,
    >(
        self,
    ) -> anyhow::Result<Spacejam<S, V>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let validator = V::try_from(self.validator.clone())
            .map_err(|_| anyhow::anyhow!("Invalid seed {:?}", self.validator))?;
        let context = Context::new(validator, S::try_from(self.db.clone())?, tx);

        // Initialize the database
        if context.runtime.storage.is_empty() {
            Self::init_storage(&self, &context)?;
        }

        // Initialize the network
        //
        // TODO: add config to the inner channel
        let network = Network::new(self.network, rx, context.keypair()).await?;
        Ok(Spacejam {
            context: Arc::new(context),
            network,
        })
    }

    /// Initialize the storage with genesis data
    fn init_storage<S: Storage, V: Validator>(
        &self,
        context: &Context<S, V>,
    ) -> anyhow::Result<()> {
        let genesis = fs::read_to_string(self.genesis.clone())
            .map_err(|_| anyhow::anyhow!("Failed to read genesis file {:?}", self.genesis))?;
        let genesis: Genesis = serde_json::from_str(&genesis)
            .map_err(|_| anyhow::anyhow!("Failed to parse genesis file {:?}", self.genesis))?;

        // insert the genesis block into database
        let block: Block = genesis.block.try_into()?;
        let recent: Vec<BlockInfo> = vec![block.header.into()];
        context
            .runtime
            .storage
            .set(key::RECENT_BLOCKS, codec::encode(&recent)?)?;

        // set up initial validators
        let validators = genesis
            .validators
            .into_iter()
            .map(Json::from_json)
            .collect::<anyhow::Result<Vec<ValidatorData>>>()?;
        let encoded = codec::encode(&validators)?;
        context
            .runtime
            .storage
            .set(key::CURRENT_VALIDATORS, encoded)?;

        // set up initial safrole state
        let safrole = Safrole {
            series: TicketsOrKeys::Keys(validators.iter().map(|v| v.bandersnatch).collect()),
            ..Default::default()
        };
        context
            .runtime
            .storage
            .set(key::SAFROLE, codec::encode(&safrole)?)?;

        // set up initial entropy
        //
        // TODO: get entropy from the genesis file
        let entropy = EntropyBuffer::default();
        context
            .runtime
            .storage
            .set(key::ENTROPY, codec::encode(&entropy)?)?;

        Ok(())
    }
}
