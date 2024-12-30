//! Spawn the node

use std::path::PathBuf;

use crate::{Config, SpaceJam};
use clap::Parser;
use score::state::Storage;

/// Spawn the node
#[derive(Default, Parser)]
pub struct Spawn {
    /// Path to the database
    pub db: Option<PathBuf>,

    /// Path to the genesis file
    pub genesis: Option<PathBuf>,
}

impl Spawn {
    /// Run the command
    pub fn run<C: Config>(&self) -> anyhow::Result<()> {
        let mut spacejam: SpaceJam<C> = SpaceJam::new(
            C::Db::open(self.db.clone().unwrap_or(PathBuf::from("chain.db")))?,
            C::Validator::default(),
        );

        // TODO: parse and load genesis file

        let mut bn = 0;
        loop {
            let block = spacejam.mine()?;
            bn += 1;
            println!("mined block #{}: {}", bn, hex::encode(block.hash()?));
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }
}
