//! The runtime of SpaceJam

pub use node::SpaceJam;
pub use runtime::{Config, Storage, Validator};

pub mod cmd;
mod node;
mod offchain;
pub mod storage;
mod utils;
pub mod validator;

/// The config of development
pub struct Development;

impl Config for Development {
    type Validator = validator::LocalValidator;
    type Storage = storage::Sled;
    type Vm = ();
    type Hook = ();
}

/// The config of production
pub struct Production;

impl Config for Production {
    type Validator = validator::LocalValidator;
    #[cfg(feature = "rocksdb")]
    type Storage = storage::RocksDB;
    #[cfg(not(feature = "rocksdb"))]
    type Storage = storage::Sled;
    type Vm = ();
    type Hook = ();
}

/// The config of test
pub struct Test;

impl Config for Test {
    type Validator = validator::LocalValidator;
    type Storage = storage::MemoryDb;
    type Vm = ();
    type Hook = ();
}
