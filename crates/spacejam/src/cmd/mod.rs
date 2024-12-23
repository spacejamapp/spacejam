//! Command line interface for spacejam

use crate::{Config, SpaceJam};
use clap::Parser;
pub use rand::Rand;
use score::state::Storage;

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
    pub fn run<C: Config>(&self) -> anyhow::Result<()> {
        match self {
            Command::Rand(rand) => rand.run(),
            Command::Spawn => {
                let mut spacejam: SpaceJam<C> =
                    SpaceJam::new(C::Db::open("chain.db")?, C::Validator::default().into());

                let mut bn = 0;
                loop {
                    let block = spacejam.mine()?;
                    bn += 1;
                    println!("mined block #{}: {}", bn, hex::encode(block.hash()?));
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        }
    }
}
