//! SpaceJam node service

use crate::Config;
use anyhow::Result;
use network::Network;
use score::Block;
use spacejam::{storage::Sled, validator::LocalValidator, Genesis, RuntimeSpec};
use std::{marker, sync::Arc};

/// Start the node service
pub async fn start<Hook: runtime::Hook + Default + Send + Sync + 'static>(
    config: &Config,
    hook: Hook,
) -> Result<()> {
    let chain = config.data.join("chain");

    // fetch the genesis block
    let genesis = config.genesis.as_ref().map(|p| p.join("genesis.json"));
    let genesis: Genesis = genesis.try_into()?;
    let block: Block = genesis.block.clone().try_into()?;

    // build the network config
    let networkcfg = network::Config {
        address: config.quic,
        bootstrap: vec![],
        genesis: block.header.hash()?,
    };

    let runtime = JadexSpec::<Hook>::runtime_with_hook(None, chain, genesis, hook).await?;
    let network = Network::<JadexSpec<Hook>>::new(networkcfg, Arc::new(runtime)).await?;
    network.spawn().await;
    Ok(())
}

/// The Jadex runtime spec
pub struct JadexSpec<Hook: runtime::Hook + Default>(marker::PhantomData<Hook>);

impl<Hook: runtime::Hook + Default + Send + Sync + 'static> runtime::Config for JadexSpec<Hook> {
    type Storage = Sled;
    type Validator = LocalValidator;
    type Vm = ();
    type Hook = Hook;
}
