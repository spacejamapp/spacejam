//! Spawn the node

use crate::node::Builder;
use clap::Parser;
use score::runtime::{Storage, Validator};
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
    pub async fn run<
        S: Storage + Send + Sync + 'static + TryFrom<PathBuf, Error = anyhow::Error>,
        V: Validator + Send + Sync + 'static + TryFrom<String> + 'static,
    >(
        &self,
    ) -> anyhow::Result<()> {
        let node = self.config.clone().build::<S, V>().await?;
        node.start(self.metrics).await
    }
}
