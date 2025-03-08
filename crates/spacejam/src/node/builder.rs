//! Configuration for the spacejam node

use crate::node::Genesis;
use network::{Event, Network};
use score::{
    block::BlockInfo,
    extrinsic::TicketsOrKeys,
    runtime::{
        storage::{BlockStorage, KVStorage},
        Head, Runtime, Storage,
    },
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
    pub async fn build<C>(self) -> anyhow::Result<(Network<C>, mpsc::UnboundedReceiver<Event>)>
    where
        C: score::runtime::Config,
        C::Validator: TryFrom<String>,
        C::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let validator = C::Validator::try_from(self.validator.clone())
            .map_err(|_| anyhow::anyhow!("Invalid seed {:?}", self.validator))?;

        // Initialize the database
        let storage = C::Storage::try_from(self.db.clone())?;
        if KVStorage::is_empty(&storage) {
            Self::init_storage(&self, &storage)?;
        }

        // Initialize the network
        //
        // TODO: add config to the inner channel
        let runtime = Arc::new(Runtime::new(validator, storage)?);
        Ok((network::Network::new(self.network, runtime, tx).await?, rx))
    }

    /// Initialize the storage with genesis data
    fn init_storage(&self, storage: &impl Storage) -> anyhow::Result<()> {
        let genesis = fs::read_to_string(self.genesis.clone())
            .map_err(|_| anyhow::anyhow!("Failed to read genesis file {:?}", self.genesis))?;
        let genesis: Genesis = serde_json::from_str(&genesis)
            .map_err(|_| anyhow::anyhow!("Failed to parse genesis file {:?}", self.genesis))?;

        // construct the finalized head
        let block: Block = genesis.block.try_into()?;
        let head = Head {
            hash: block.hash()?,
            slot: block.header.slot,
        };
        storage.save_block(&block)?;
        storage.set_finalized(&head)?;

        // construct the recent blocks
        let recent: Vec<BlockInfo> = vec![block.header.into()];
        storage.set(key::RECENT_BLOCKS, codec::encode(&recent)?)?;

        // set up initial validators
        let validators = genesis
            .validators
            .into_iter()
            .map(Json::from_json)
            .collect::<anyhow::Result<Vec<ValidatorData>>>()?;
        let encoded = codec::encode(&validators)?;
        storage.set(key::PREVIOUS_VALIDATORS, encoded.clone())?;
        storage.set(key::CURRENT_VALIDATORS, encoded.clone())?;
        storage.set(key::NEXT_VALIDATORS, encoded)?;

        // set up initial safrole state
        let safrole = Safrole {
            series: TicketsOrKeys::Keys(validators.iter().map(|v| v.bandersnatch).collect()),
            ..Default::default()
        };
        storage.set(key::SAFROLE, codec::encode(&safrole)?)?;

        // set up initial entropy
        //
        // TODO: get entropy from the genesis file
        let entropy = EntropyBuffer::default();
        storage.set(key::ENTROPY, codec::encode(&entropy)?)
    }
}
