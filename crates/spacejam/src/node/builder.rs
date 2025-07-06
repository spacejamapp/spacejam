//! Configuration for the spacejam node

use crate::{
    chain::{ParsedSpec, Spec},
    node::{spec, SpaceJam},
};
use network::Network;
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};

/// Spacejam node builder
#[derive(Clone)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Builder {
    /// The genesis path
    #[cfg_attr(feature = "cmd", arg(long, env = "CHAIN"))]
    pub chain: Option<PathBuf>,

    /// The data path
    #[cfg_attr(feature = "cmd", arg(short, long, default_value_t = default::data_path(), env = "DATA_PATH"))]
    pub data_path: String,

    /// Whether running in dev mode
    #[cfg_attr(feature = "cmd", arg(long, env = "DEV"))]
    pub dev: bool,

    /// Whether running in light mode
    #[cfg_attr(feature = "cmd", arg(long, env = "LIGHT"))]
    pub light: bool,

    /// The network configuration
    #[cfg_attr(feature = "cmd", command(flatten))]
    pub network: network::Config,

    /// Whether pruning the data directory before running
    #[cfg_attr(feature = "cmd", arg(short, long, env = "PRUNE"))]
    pub prune: bool,

    /// The RPC address
    #[cfg_attr(
        feature = "cmd",
        arg(short, long, default_value = "0.0.0.0:6789", env = "RPC")
    )]
    pub rpc: SocketAddr,

    /// The seed of the validator
    #[cfg_attr(feature = "cmd", arg(long, env = "VALIDATOR"))]
    pub validator: Option<String>,
}

impl Builder {
    /// Build the node
    pub async fn build<C: spec::RuntimeSpecSelf>(mut self) -> anyhow::Result<SpaceJam<C>> {
        let genesis = self.genesis()?;
        let data = self.data(&genesis)?;

        // prepare the runtime
        let runtime = C::runtime(self.validator.as_deref(), data, genesis).await?;
        self.build_with_runtime(runtime).await
    }

    /// Build the node with hook
    pub async fn build_with_hook<C: spec::RuntimeSpec>(
        mut self,
        hook: C::Hook,
    ) -> anyhow::Result<SpaceJam<C>> {
        let genesis = self.genesis()?;
        let data = self.data(&genesis)?;

        // prepare the runtime
        let runtime = C::runtime_with_hook(self.validator.as_deref(), data, genesis, hook).await?;
        self.build_with_runtime(runtime).await
    }

    /// Build the node with hook
    pub async fn build_with_runtime<C: spec::RuntimeSpec>(
        &self,
        runtime: runtime::Runtime<C>,
    ) -> anyhow::Result<SpaceJam<C>> {
        if self.dev {
            return Ok(SpaceJam::Dev(spec::Dev {
                runtime,
                rpc: self.rpc,
            }));
        }

        let mut network = Network::new(self.network.clone(), Arc::new(runtime)).await?;
        if self.light {
            return Ok(SpaceJam::Light(spec::Light {
                network,
                rpc: self.rpc,
            }));
        }

        network.broadcast = true;
        Ok(SpaceJam::Validating(spec::Validating(network)))
    }

    fn genesis(&mut self) -> anyhow::Result<ParsedSpec> {
        let parsed_genesis = if let Some(genesis) = &self.chain {
            serde_json::from_slice(fs::read(genesis)?.as_slice())?
        } else {
            Spec::dev()
        }
        .parse()?;

        // apply config from the spec file
        //
        // TODO: handle bootnode and peer id
        self.network.genesis = parsed_genesis.genesis_header.hash()?;

        Ok(parsed_genesis)
    }

    fn data(&self, genesis: &ParsedSpec) -> anyhow::Result<PathBuf> {
        let data = PathBuf::from(&self.data_path).join(genesis.id.to_string());

        if self.prune && data.exists() {
            fs::remove_dir_all(&data)?;
        }

        if !data.exists() {
            fs::create_dir_all(&data)?;
        }
        Ok(data)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            chain: None,
            data_path: default::data_path(),
            rpc: SocketAddr::from(([0, 0, 0, 0], 6789)),
            network: network::Config::default(),
            prune: false,
            validator: None,
            dev: false,
            light: false,
        }
    }
}

mod default {
    /// The default data path
    pub fn data_path() -> String {
        dirs::data_dir()
            .unwrap_or_default()
            .join("spacejam")
            .to_string_lossy()
            .to_string()
    }
}
