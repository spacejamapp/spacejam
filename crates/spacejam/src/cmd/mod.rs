//! Command line interface for spacejam

use crate::{
    node::Genesis,
    node::{spec, Builder},
};
use clap::Parser;
use runtime::Storage;
use score::block::BlockJson;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The command line interface for spacejam
#[derive(Parser)]
pub enum Command {
    /// Generate random data
    Genesis,

    /// Import a block
    Import {
        /// The database path
        #[arg(long)]
        db: PathBuf,

        /// The genesis path
        #[arg(long)]
        genesis: Option<PathBuf>,

        /// The path of block file
        #[arg(long)]
        block: PathBuf,
    },

    /// Start the SpaceJam node
    Spawn(Box<Builder>),

    /// Print the state
    State {
        /// The database path
        #[arg(long)]
        db: PathBuf,

        /// The hash of the block header
        #[arg(long)]
        hash: Option<String>,
    },
}

impl Command {
    /// Run the command
    pub async fn run<C>(self) -> anyhow::Result<()>
    where
        C: spec::RuntimeSpec,
        <C as runtime::Config>::Hook: Default,
    {
        match self {
            Command::Genesis => Self::genesis(),
            Command::Import { db, block, genesis } => {
                Self::import::<C>(&db, &block, genesis.as_deref()).await
            }
            Command::State { db, hash } => Self::state::<C>(&db, hash.as_deref()),
            Command::Spawn(spawn) => spawn.build::<C>().await?.start().await,
        }
    }

    async fn import<C>(db: &Path, block: &Path, genesis: Option<&Path>) -> anyhow::Result<()>
    where
        C: spec::RuntimeSpec,
        <C as runtime::Config>::Hook: Default,
    {
        let genesis = genesis.map(|p| p.to_path_buf()).try_into()?;
        let runtime = C::runtime(None, db.to_path_buf(), genesis).await?;

        let block = fs::read_to_string(block)?;
        let block: BlockJson = serde_json::from_str(&block)?;
        runtime.finalize(block.try_into()?).await?;
        Ok(())
    }

    fn state<C>(db: &Path, _hash: Option<&str>) -> anyhow::Result<()>
    where
        C: runtime::Config,
        C::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
    {
        let storage = C::Storage::try_from(db.to_path_buf())?;
        let state = storage.state()?;
        println!("{}", serde_json::to_string_pretty(&state)?);
        Ok(())
    }

    fn genesis() -> anyhow::Result<()> {
        let genesis = Genesis::default();
        println!("{}", serde_json::to_string_pretty(&genesis)?);
        Ok(())
    }
}
