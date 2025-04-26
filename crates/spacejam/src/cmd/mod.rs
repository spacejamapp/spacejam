//! Command line interface for spacejam

use crate::{node::spec, node::Genesis};
use clap::Parser;
use runtime::Storage;
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
    pub async fn run<C: spec::RuntimeSpec>(&self) -> anyhow::Result<()> {
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
        let genesis = Genesis::default();
        println!("{}", serde_json::to_string_pretty(&genesis)?);
        Ok(())
    }
}
