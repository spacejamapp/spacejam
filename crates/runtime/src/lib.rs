//! Runtime utilities of SpaceJam

use pvm::Pvm;
use score::{extrinsic::TicketEnvelope, BandersnatchPublic};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub use {
    account::{Account, Accounts},
    chain::{Chain, Grid, Lookup},
    grandpa::{Grandpa, Handshake},
    hook::Hook,
    pool::Pool,
    storage::Storage,
    validator::Validator,
};

mod account;
pub mod chain;
mod grandpa;
mod hook;
mod pool;
pub mod storage;
pub mod tx;
mod validator;
mod work;

/// Runtime of SpaceJam
#[derive(Clone)]
pub struct Runtime<C: Config> {
    /// The chain of blocks
    ///
    /// This should never being used directly, use the `chain` method instead.
    _chain: Arc<RwLock<Chain<C>>>,

    /// The validator of SpaceJam
    pub validator: C::Validator,

    /// The hook of SpaceJam
    pub hook: C::Hook,

    /// The extrinsic pool of SpaceJam
    pub expool: Arc<Mutex<Pool>>,

    /// The received tickets per epoch
    pub tickets: Arc<Mutex<Vec<(u32, TicketEnvelope)>>>,
}

impl<C: Config> Runtime<C> {
    /// Create a new runtime with a grandpa instance
    pub fn new(validator: C::Validator, storage: C::Storage, hook: C::Hook) -> Self {
        let storage = Arc::new(storage);
        Self {
            validator,
            _chain: Arc::new(RwLock::new(Chain::new(storage.clone()))),
            hook,
            expool: Default::default(),
            tickets: Default::default(),
        }
    }

    /// Finalize the chain
    pub async fn finalize(&self) -> anyhow::Result<()> {
        for (block, diff) in self._chain.write().await.finalize()? {
            self.hook.on_diff(block.header.hash()?, diff).await?;
            self.hook.on_finalized_block(block).await?;
        }

        Ok(())
    }

    /// Get the bandersnatch public key of the local validator
    pub fn me(&self) -> BandersnatchPublic {
        self.validator.bandersnatch_public_key()
    }
}

/// The configuration of the runtime
pub trait Config: Send + Sync + 'static {
    /// The storage of the runtime
    type Storage: Storage + Send + Sync + 'static;

    /// The validator of the runtime
    type Validator: Validator + Send + Sync + 'static;

    /// The virtual machine of the runtime
    type Vm: Pvm + Send + Sync + 'static;

    /// The hook of the runtime
    type Hook: Hook + Send + Sync + 'static;
}

impl Config for () {
    type Storage = storage::MemoryDb;
    type Validator = crypto::ed25519::KeyPair;
    type Vm = ();
    type Hook = ();
}
