//! Command line interface for spacejam

use crate::{node::Genesis, validator::LocalValidator};
use clap::Parser;
use runtime::{Storage, Validator};
use score::{block::header::EValidator, Block};
use spacejson::Json;
pub use spawn::Spawn;
use std::path::{Path, PathBuf};

mod spawn;

/// The command line interface for spacejam
#[derive(Parser)]
pub enum Command {
    /// Generate random data
    Genesis,

    /// Print the state
    State {
        /// The database path
        #[arg(long)]
        db: PathBuf,
    },

    /// Start the SpaceJam node
    Spawn(Box<Spawn>),
}

impl Command {
    /// Run the command
    pub async fn run<C>(&self) -> anyhow::Result<()>
    where
        C: runtime::Config,
        C::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
        C::Validator: TryFrom<String>,
    {
        match self {
            Command::Genesis => self.genesis(),
            Command::State { db } => self.state::<C>(db),
            Command::Spawn(spawn) => spawn.run::<C>().await,
        }
    }

    fn state<C>(&self, db: &Path) -> anyhow::Result<()>
    where
        C: runtime::Config,
        C::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
    {
        let storage = C::Storage::try_from(db.to_path_buf())?;
        let state = storage.state()?;
        println!("{}", serde_json::to_string_pretty(&state)?);
        Ok(())
    }

    fn genesis(&self) -> anyhow::Result<()> {
        let mut validators = Vec::new();
        let mut bkeys = [EValidator::default(); score::VALIDATORS_COUNT as usize];
        for i in 0..score::VALIDATORS_COUNT {
            let validator = LocalValidator::from([i as u8; 32]);
            let data = validator.data();
            bkeys[i as usize] = EValidator {
                bandersnatch: data.bandersnatch,
                ed25519: data.ed25519,
            };
            validators.push(data.to_json());
        }

        // print the genesis block
        let genesis = Block::genesis(bkeys);
        let genesis = Genesis {
            block: genesis.to_json(),
            validators,
        };

        println!("{}", serde_json::to_string_pretty(&genesis)?);
        Ok(())
    }
}
