//! Configuration for the spacejam node

use crate::{
    chain,
    node::{spec, SpaceJam},
};
use network::Network;
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};

/// Spacejam node builder
#[derive(Clone)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Builder {
    /// The genesis path
    #[cfg_attr(feature = "cmd", arg(long))]
    chain: Option<PathBuf>,

    /// The data path
    #[cfg_attr(feature = "cmd", arg(short, long, default_value_t = default::data_path()))]
    data_path: String,

    /// Whether running in dev mode
    #[cfg_attr(feature = "cmd", arg(long))]
    dev: bool,

    /// Whether running in light mode
    #[cfg_attr(feature = "cmd", arg(long))]
    light: bool,

    /// The network configuration
    #[cfg_attr(feature = "cmd", command(flatten))]
    network: network::Config,

    /// The RPC address
    #[cfg_attr(feature = "cmd", arg(short, long, default_value = "0.0.0.0:6789"))]
    rpc: SocketAddr,

    /// The seed of the validator
    #[cfg_attr(feature = "cmd", arg(long))]
    validator: Option<String>,
}

impl Builder {
    /// Build the node
    pub async fn build<C: spec::RuntimeSpecSelf>(mut self) -> anyhow::Result<SpaceJam<C>> {
        let genesis = if let Some(genesis) = self.chain {
            serde_json::from_slice(fs::read(&genesis)?.as_slice())?
        } else {
            chain::Spec::dev()
        }
        .parse()?;

        // apply config from the spec file
        //
        // TODO: handle bootnode and peer id
        self.network.genesis = genesis.genesis_header.hash()?;

        // prepare the runtime
        let data = {
            let data = PathBuf::from(self.data_path).join(genesis.id.to_string());
            if !data.exists() {
                fs::create_dir_all(&data)?;
            }
            data
        };

        let runtime = C::runtime(self.validator.as_deref(), data, genesis).await?;
        if self.dev {
            return Ok(SpaceJam::Dev(spec::Dev {
                runtime,
                rpc: self.rpc,
            }));
        }

        let network = Network::new(self.network.clone(), Arc::new(runtime)).await?;
        if self.light {
            return Ok(SpaceJam::Light(spec::Light {
                network,
                rpc: self.rpc,
            }));
        }

        Ok(SpaceJam::Validating(spec::Validating(network)))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            chain: None,
            data_path: default::data_path(),
            rpc: SocketAddr::from(([0, 0, 0, 0], 6789)),
            network: network::Config::default(),
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
