//! Node specification

use crate::chain;
use runtime::{storage::KVStorage, Runtime, Validator};
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

    /// Build the runtime
    fn runtime(
        validator: Option<&str>,
        db: PathBuf,
        genesis: chain::ParsedSpec,
    ) -> impl std::future::Future<Output = anyhow::Result<Runtime<Self>>> + Send
    where
        <Self as runtime::Config>::Hook: Default,
    {
        async move { Self::runtime_with_hook(validator, db, genesis, Self::Hook::default()).await }
    }

    /// Build the runtime with a hook
    fn runtime_with_hook(
        validator: Option<&str>,
        db: PathBuf,
        genesis: chain::ParsedSpec,
        hook: Self::Hook,
    ) -> impl std::future::Future<Output = anyhow::Result<Runtime<Self>>> + Send {
        async move {
            let validator = Self::validator(validator)?;
            let storage = Self::storage(db)?;
            let runtime = Runtime::new(validator, storage, hook);

            // Initialize the database
            if KVStorage::is_empty(&runtime.storage) {
                runtime
                    .import_genesis(genesis.genesis_header, &genesis.genesis_state)
                    .await?;
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
{
}

/// Runtime configuration for nodes in the current crate
pub trait RuntimeSpecSelf: RuntimeSpec<Hook: Default> {}

impl<T: RuntimeSpec<Hook: Default>> RuntimeSpecSelf for T {}
