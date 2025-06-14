//! Runtime utilities of SpaceJam

use pvm::Pvm;
use score::BandersnatchPublic;
use std::sync::Arc;
use tokio::sync::RwLock;
pub use {
    grandpa::{Grandpa, Handshake},
    hook::Hook,
    pool::Pool,
    storage::Storage,
    validator::Validator,
};

pub mod account;
mod ext;
mod grandpa;
mod hook;
mod pool;
pub mod storage;
pub mod tx;
mod validator;

/// Runtime of SpaceJam
#[derive(Clone)]
pub struct Runtime<C: Config> {
    /// The validator of SpaceJam
    pub validator: C::Validator,

    /// The storage of SpaceJam
    pub storage: Arc<C::Storage>,

    /// The hook of SpaceJam
    pub hook: C::Hook,

    /// The extrinsic pool of SpaceJam
    pub expool: Pool,

    /// The grandpa of SpaceJam
    pub grandpa: Arc<RwLock<Grandpa>>,
}

impl<C: Config> Runtime<C> {
    /// Create a new runtime with a grandpa instance
    pub fn new(validator: C::Validator, storage: C::Storage, hook: C::Hook) -> Self {
        Self {
            validator,
            storage: Arc::new(storage),
            hook,
            expool: Default::default(),
            grandpa: Arc::new(RwLock::new(Default::default())),
        }
    }

    /// Get the local validator
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
