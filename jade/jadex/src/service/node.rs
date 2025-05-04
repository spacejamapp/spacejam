//! SpaceJam node service

use crate::Config;
use anyhow::Result;
use network::Network;
use runtime::Runtime;
use score::Block;
use spacejam::{storage::Sled, validator::LocalValidator, Genesis, RuntimeSpec};
use std::sync::Arc;

/// The SpaceJam node service
pub struct Node<Hook: runtime::Hook + Default + Send + Sync + 'static> {
    _hook: Hook,
}

impl<Hook: runtime::Hook + Default + Send + Sync + 'static> Node<Hook> {
    /// initialize a new node
    pub fn new(_hook: Hook) -> Self {
        Self { _hook }
    }

    /// start the node service
    pub async fn start(&self, config: &Config) -> Result<()> {
        self.network(config).await?.spawn().await;
        Ok(())
    }

    /// initialize a new network
    async fn network(&self, config: &Config) -> Result<Network<JadexSpec<Hook>>> {
        let genesis = config.genesis.as_ref().map(|p| p.join("genesis.json"));
        let genesis: Genesis = genesis.try_into()?;
        let block: Block = genesis.block.try_into()?;
        let networkcfg = network::Config {
            address: config.quic,
            bootstrap: vec![],
            genesis: block.header.hash()?,
        };
        let runtime = self.runtime(config).await?;
        let network = Network::<JadexSpec<Hook>>::new(networkcfg, Arc::new(runtime)).await?;
        Ok(network)
    }

    /// Build the runtime
    async fn runtime(&self, config: &Config) -> Result<Runtime<JadexSpec<Hook>>> {
        let chain = config.data.join("chain");
        let genesis = config.genesis.as_ref().map(|p| p.join("genesis.json"));
        JadexSpec::<Hook>::runtime(None, chain, genesis.try_into()?).await
    }
}

/// The Jadex runtime spec
pub struct JadexSpec<Hook: runtime::Hook + Default> {
    _hook: Hook,
}

impl<Hook: runtime::Hook + Default + Send + Sync + 'static> runtime::Config for JadexSpec<Hook> {
    type Storage = Sled;
    type Validator = LocalValidator;
    type Vm = ();
    type Hook = Hook;
}
