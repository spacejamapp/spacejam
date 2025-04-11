//! Runtime utilities of SpaceJam

use author::Author;
use importer::Importer;
use pvm::Pvm;
use std::sync::Arc;
use tokio::sync::RwLock;
pub use {
    grandpa::{Grandpa, Handshake, Head},
    pool::Pool,
    storage::Storage,
    validator::Validator,
};

mod author;
mod grandpa;
mod importer;
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
    pub storage: C::Storage,

    /// The extrinsic pool of SpaceJam
    pub expool: Pool,

    /// The grandpa of SpaceJam
    pub grandpa: Arc<RwLock<Grandpa>>,
}

impl<C: Config> Runtime<C> {
    /// Create a new runtime with a grandpa instance
    pub fn new(validator: C::Validator, storage: C::Storage) -> Self {
        Self {
            validator,
            storage,
            expool: Default::default(),
            grandpa: Arc::new(RwLock::new(Default::default())),
        }
    }

    /// Get the authoring context
    pub fn author(&self) -> Author<C> {
        Author::new(self)
    }

    /// Get the importer
    pub fn importer(&self) -> Importer<C> {
        Importer::new(self)
    }
}

/// The configuration of the runtime
///
/// TODO: introduce hooks for the runtime.
pub trait Config: Send + Sync + 'static {
    /// The storage of the runtime
    type Storage: Storage + Send + Sync + 'static;

    /// The validator of the runtime
    type Validator: Validator + Send + Sync + 'static;

    /// The virtual machine of the runtime
    type Vm: Pvm + Send + Sync + 'static;
}

impl Config for () {
    type Storage = storage::MemoryDb;
    type Validator = crypto::ed25519::KeyPair;
    type Vm = ();
}
