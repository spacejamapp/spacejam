//! Command `rand` for spacejam

use anyhow::Result;
use clap::Parser;
use score::block::Block;

/// The `rand` command
///
/// Which generates random test data.
#[derive(Parser)]
pub enum Rand {
    /// Generate random genesis block in json format
    #[command(name = "genesis")]
    Genesis,
}

impl Rand {
    /// Run the `rand` command
    pub fn run(&self) -> Result<()> {
        match self {
            Rand::Genesis => self.genesis(),
        }
    }

    fn genesis(&self) -> Result<()> {
        let block = Block::default();
        println!("{}", serde_json::to_string_pretty(&block)?);
        Ok(())
    }
}
