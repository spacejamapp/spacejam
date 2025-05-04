//! Node specification

use super::Genesis;
use runtime::{storage::KVStorage, Runtime, Validator};
use score::{safrole::ValidatorData, Block};
use std::path::PathBuf;
pub use {dev::Dev, light::Light, validating::Validating};

mod dev;
mod light;
mod validating;

/// SpaceJam node interface
pub trait NodeSpec {
    /// Start the node
    async fn start(self) -> anyhow::Result<()>;
}

/// Runtime configuration
pub trait RuntimeSpec:
    runtime::Config<
        Validator: TryFrom<String, Error = anyhow::Error>,
        Storage: TryFrom<PathBuf, Error = anyhow::Error>,
        Hook: Default,
    > + Sized
{
    /// Build the validator
    fn validator(mb_validator: Option<&str>) -> anyhow::Result<Self::Validator> {
        if let Some(raw) = mb_validator {
            Self::Validator::try_from(raw.to_string())
        } else {
            Ok(Self::Validator::random())
        }
    }

    /// Build the storage
    fn storage(path: PathBuf) -> anyhow::Result<Self::Storage> {
        Self::Storage::try_from(path)
    }

    /// Build the hook
    fn hook() -> Self::Hook {
        Default::default()
    }

    /// Build the runtime
    fn runtime(
        validator: Option<&str>,
        db: PathBuf,
        genesis: Genesis,
    ) -> impl std::future::Future<Output = anyhow::Result<Runtime<Self>>> + Send {
        async move {
            let validator = Self::validator(validator)?;
            let storage = Self::storage(db)?;
            let hook = Self::hook();
            let runtime = Runtime::new(validator, storage, hook);

            // Initialize the database
            //
            // TODO: validate the genesis block matches the storage if not empty
            if KVStorage::is_empty(&runtime.storage) {
                let block = Block::try_from(genesis.block)?;
                let validators = genesis
                    .validators
                    .into_iter()
                    .map(ValidatorData::try_from)
                    .collect::<anyhow::Result<Vec<_>>>()?;

                runtime.import_genesis(block, &validators).await?;
            }
            Ok(runtime)
        }
    }
}

impl<T> RuntimeSpec for T
where
    T: runtime::Config,
    T::Validator: TryFrom<String, Error = anyhow::Error>,
    T::Storage: TryFrom<PathBuf, Error = anyhow::Error>,
    T::Hook: Default,
{
}
