//! Command line interface for spacejam

use crate::Config;
use clap::Parser;
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
    Spawn(Spawn),
}

impl Command {
    /// Run the command
    pub async fn run<C: Config + 'static>(&self) -> anyhow::Result<()> {
        match self {
            Command::Rand(rand) => rand.run(),
            Command::Spawn(spawn) => spawn.run::<C>().await,
        }
    }
}

impl Default for Command {
    fn default() -> Self {
        Command::Spawn(Spawn::default())
    }
}
