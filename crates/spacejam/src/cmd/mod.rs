//! Command line interface for spacejam

use clap::Parser;
pub use rand::Rand;

mod rand;

/// The command line interface for spacejam
#[derive(Parser, Default)]
pub enum Command {
    /// Generate random data
    #[command(subcommand)]
    Rand(Rand),

    /// Start the SpaceJam node
    #[default]
    Spawn,
}

impl Command {
    /// Run the command
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Rand(rand) => rand.run(),
            Command::Spawn => Ok(()),
        }
    }
}
