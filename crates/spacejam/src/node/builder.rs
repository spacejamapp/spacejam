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
    /// The database path
    #[cfg_attr(feature = "cmd", arg(long, default_value = "spacejam.db"))]
    db: PathBuf,

    /// Whether running in dev mode
    #[cfg_attr(feature = "cmd", arg(long))]
    dev: bool,

    /// Whether running in light mode
    #[cfg_attr(feature = "cmd", arg(long))]
    light: bool,

    /// The genesis path
    #[cfg_attr(feature = "cmd", arg(long))]
    genesis: Option<PathBuf>,

    /// The metrics address
    #[cfg_attr(feature = "cmd", arg(short, long, default_value = "0.0.0.0:0"))]
    metrics: SocketAddr,

    /// The RPC address
    #[cfg_attr(feature = "cmd", arg(short, long, default_value = "0.0.0.0:6789"))]
    rpc: SocketAddr,

    /// The network configuration
    #[cfg_attr(feature = "cmd", command(flatten))]
    network: network::Config,

    /// The seed of the validator
    ///
    /// TODO: make this field optional, if not provided, the node will not be a validator.
    #[cfg_attr(feature = "cmd", arg(long))]
    validator: Option<String>,
}

impl Builder {
    /// Build the node
    pub async fn build<C: spec::RuntimeSpecSelf>(mut self) -> anyhow::Result<SpaceJam<C>> {
        let genesis = if let Some(genesis) = self.genesis {
            serde_json::from_slice(fs::read(&genesis)?.as_slice())?
        } else {
            chain::Spec::dev()
        }
        .parse()?;

        // apply config from the spec file
        self.network.genesis = genesis.genesis_header.hash()?;
        if self.network.bootnodes.is_empty() {
            self.network.bootnodes = genesis.bootnodes.clone();
        }

        // prepare the runtime
        let runtime = C::runtime(self.validator.as_deref(), self.db.clone(), genesis).await?;
        if self.dev {
            return Ok(SpaceJam::Dev(spec::Dev {
                runtime,
                rpc: self.rpc,
                metrics: self.metrics,
            }));
        }

        let network = Network::new(self.network.clone(), Arc::new(runtime)).await?;
        if self.light {
            return Ok(SpaceJam::Light(spec::Light {
                network,
                rpc: self.rpc,
                metrics: self.metrics,
            }));
        }

        Ok(SpaceJam::Validating(spec::Validating(network)))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            db: PathBuf::from("spacejam.db"),
            genesis: None,
            metrics: SocketAddr::from(([0, 0, 0, 0], 0)),
            rpc: SocketAddr::from(([0, 0, 0, 0], 6789)),
            network: network::Config::default(),
            validator: None,
            dev: false,
            light: false,
        }
    }
}
