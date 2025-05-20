//! Command line interface for spacejam

use crate::node::{spec, Builder};
use clap::Parser;

/// The command line interface for spacejam
#[derive(Parser)]
pub enum Command {
    /// Start the SpaceJam node
    Run(Box<Builder>),
}

impl Command {
    /// Run the command
    pub async fn run<C: spec::RuntimeSpecSelf>(self) -> anyhow::Result<()> {
        match self {
            Command::Run(run) => run.build::<C>().await?.start().await,
        }
    }
}
