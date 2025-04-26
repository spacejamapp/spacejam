//! Configuration for the spacejam node

use crate::node::{spec, SpaceJam};
use network::Network;
use runtime::Runtime;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

/// Spacejam node builder
#[derive(Clone)]
#[cfg_attr(feature = "cmd", derive(clap::Parser))]
pub struct Builder {
    /// The database path
    #[cfg_attr(feature = "cmd", arg(long, default_value = "spacejam.db"))]
    db: PathBuf,

    /// Wether running in dev mode
    #[cfg_attr(feature = "cmd", arg(long))]
    dev: bool,

    /// Wether running in light mode
    #[cfg_attr(feature = "cmd", arg(long))]
    light: bool,

    /// The genesis path
    #[cfg_attr(feature = "cmd", arg(long, default_value = "genesis.json"))]
    genesis: PathBuf,

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
    pub async fn build<C: spec::RuntimeSpec>(self) -> anyhow::Result<SpaceJam<C>> {
        let runtime = self.runtime::<C>().await?;
        if self.dev {
            return Ok(SpaceJam::Dev(spec::Dev(runtime)));
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

    /// Build the runtime
    async fn runtime<C: spec::RuntimeSpec>(&self) -> anyhow::Result<Runtime<C>> {
        C::runtime(
            self.validator.as_deref(),
            self.db.clone(),
            self.genesis.clone(),
        )
        .await
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            db: PathBuf::from("spacejam.db"),
            genesis: PathBuf::from("genesis.json"),
            metrics: SocketAddr::from(([0, 0, 0, 0], 0)),
            rpc: SocketAddr::from(([0, 0, 0, 0], 6789)),
            network: network::Config::default(),
            validator: None,
            dev: false,
            light: false,
        }
    }
}
