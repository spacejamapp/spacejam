//! Spawn the node

use crate::node::{self, Builder};
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};

/// Spawn the node
#[derive(Parser)]
pub struct Spawn {
    /// Metrics address
    #[arg(short, long, default_value = "0.0.0.0:0")]
    pub metrics: SocketAddr,

    /// The configuration
    #[command(flatten)]
    pub config: Builder,
}

impl Spawn {
    /// Run the command
    pub async fn run<C>(&self) -> anyhow::Result<()>
    where
        C: runtime::Config,
        C::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
        C::Validator: TryFrom<String>,
        C::Hook: Default,
    {
        let (network, rx) = self.config.clone().build::<C>().await?;
        node::start(network, rx, self.metrics).await
    }
}
