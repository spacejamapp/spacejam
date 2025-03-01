//! Command line interface for spacejam

use clap::Parser;
use std::path::PathBuf;
pub use {rand::Rand, spawn::Spawn};

mod rand;
mod spawn;

/// The command line interface for spacejam
#[derive(Parser)]
pub enum Command {
    /// Generate random data
    #[command(subcommand)]
    Rand(Rand),

    /// Start the SpaceJam node
    Spawn(Box<Spawn>),
}

impl Command {
    /// Run the command
    pub async fn run<C>(&self) -> anyhow::Result<()>
    where
        C: score::runtime::Config,
        C::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
        C::Validator: TryFrom<String>,
    {
        match self {
            Command::Rand(rand) => rand.run(),
            Command::Spawn(spawn) => spawn.run::<C>().await,
        }
    }
}
