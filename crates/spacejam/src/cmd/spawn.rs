//! Spawn the node

use crate::node::{spec::RuntimeSpec, Builder};
use clap::Parser;

/// Spawn the node
#[derive(Parser)]
pub struct Spawn {
    /// The configuration
    #[command(flatten)]
    pub config: Builder,
}

impl Spawn {
    /// Run the command
    pub async fn run<C: RuntimeSpec>(&self) -> anyhow::Result<()> {
        self.config.clone().build::<C>().await?.start().await
    }
}
