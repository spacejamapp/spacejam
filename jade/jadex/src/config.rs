//! Config for the JAM index

use anyhow::Result;
use clap::Parser;
use network::Network;
use runtime::Runtime;
use score::Block;
use spacejam::{storage::Sled, validator::LocalValidator, Genesis, RuntimeSpec};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

/// Config for the JAM index
#[derive(Parser)]
pub struct Config {
    /// The postgres database url
    #[clap(long, default_value = "postgres://postgres@postgres")]
    pub postgres: String,

    /// The path to the chain data
    #[clap(long)]
    pub data: PathBuf,

    /// The path to the genesis file
    #[clap(long)]
    pub genesis: Option<PathBuf>,

    /// The graphql server address
    #[clap(long, default_value = "0.0.0.0:8080")]
    pub graphql: SocketAddr,

    /// The address for the quic transport
    #[clap(long, default_value = "0.0.0.0:0")]
    pub quic: SocketAddr,
}

impl Config {
    /// Start the service
    ///
    /// TODO:
    ///
    /// 1. add graphQL service and sqlx service
    /// 2. inject hook for storing the data
    pub async fn start(&self) -> Result<()> {
        let network = self.network().await?;
        network.spawn().await;
        Ok(())
    }

    async fn network(&self) -> Result<Network<Self>> {
        let genesis = self.genesis.as_ref().map(|p| p.join("genesis.json"));
        let genesis: Genesis = genesis.try_into()?;
        let block: Block = genesis.block.try_into()?;

        let networkcfg = network::Config {
            address: self.quic,
            bootstrap: vec![],
            genesis: block.header.hash()?,
        };
        let runtime = self.runtime().await?;
        let network = Network::<Self>::new(networkcfg, Arc::new(runtime)).await?;
        Ok(network)
    }

    /// Build the runtime
    async fn runtime(&self) -> Result<Runtime<Self>> {
        let chain = self.data.join("chain");
        let genesis = self.genesis.as_ref().map(|p| p.join("genesis.json"));
        <Self as RuntimeSpec>::runtime(None, chain, genesis.try_into()?).await
    }
}

impl runtime::Config for Config {
    type Storage = Sled;
    type Validator = LocalValidator;
    type Vm = ();
    type Hook = ();
}
