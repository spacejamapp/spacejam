//! Command line interface for spacejam

use crate::{storage::sled::Sled, SpaceJam};
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
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Rand(rand) => rand.run(),
            Command::Spawn => {
                let db = Sled::open("chain.db")?;
                let mut spacejam = SpaceJam::new(db);

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
