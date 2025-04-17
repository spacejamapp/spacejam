//! Configuration for the spacejam node

use crate::node::Genesis;
use network::Network;
use runtime::{storage::KVStorage, Runtime};
use score::{safrole::ValidatorData, Block};
use std::{fs, path::PathBuf, sync::Arc};

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
    pub async fn build<C>(self) -> anyhow::Result<Network<C>>
    where
        C: runtime::Config,
        C::Validator: TryFrom<String>,
        C::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
        C::Hook: Default,
    {
        let validator = C::Validator::try_from(self.validator.clone())
            .map_err(|_| anyhow::anyhow!("Invalid seed {:?}", self.validator))?;

        // Initialize the runtime
        //
        // TODO: add config to the inner channel
        let storage = C::Storage::try_from(self.db.clone())?;
        let runtime = Arc::new(Runtime::new(validator, storage, Default::default()));

        // Initialize the database
        let importer = runtime.importer();
        if KVStorage::is_empty(&runtime.storage) {
            let genesis: Genesis = serde_json::from_slice(fs::read(self.genesis)?.as_slice())?;
            let block = Block::try_from(genesis.block)?;
            let validators = genesis
                .validators
                .into_iter()
                .map(ValidatorData::try_from)
                .collect::<anyhow::Result<Vec<_>>>()?;

            importer.import_genesis(block, &validators).await?;
        }

        // Initialize the network
        network::Network::new(self.network, runtime).await
    }
}
