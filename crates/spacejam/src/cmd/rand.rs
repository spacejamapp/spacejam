//! Command `rand` for spacejam

use crate::{node::Genesis, validator::LocalValidator};
use anyhow::Result;
use clap::Parser;
use score::{validator::Validator, VALIDATORS_COUNT};
use spacejson::Json;

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
        let mut genesis = Genesis::default();
        for i in 0..VALIDATORS_COUNT {
            let validator = LocalValidator::from([i as u8; 32]);
            genesis.validators.push(validator.data().to_json());
        }
        println!("{}", serde_json::to_string_pretty(&genesis)?);
        Ok(())
    }
}
