//! Command line interface for spacejam

use clap::Parser;
use score::runtime::{Storage, Validator};
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
    pub async fn run<
        S: Storage + 'static + TryFrom<PathBuf, Error = anyhow::Error>,
        V: Validator + From<[u8; 32]> + TryFrom<String> + 'static,
    >(
        &self,
    ) -> anyhow::Result<()> {
        match self {
            Command::Rand(rand) => rand.run(),
            Command::Spawn(spawn) => spawn.run::<S, V>().await,
        }
    }
}
