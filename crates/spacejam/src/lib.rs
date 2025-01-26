//! The runtime of SpaceJam

pub use score::{validator::Validator, Config};

pub mod cmd;
pub mod metrics;
pub mod storage;
pub mod validator;

/// The runtime of SpaceJam
///
/// NOTE: need to get avoid of too many generic parameters...
pub struct SpaceJam<C: Config> {
    /// The validator of SpaceJam
    pub validator: C::Validator,

    /// The storage of SpaceJam
    pub db: C::Db,
}

impl<C: Config> SpaceJam<C> {
    /// Initialize the chain with the given database.
    pub fn new(db: C::Db, validator: C::Validator) -> Self {
        Self { validator, db }
    }
}

impl<C: Config> network::Context for SpaceJam<C> {
    fn import_block(&self, block: Vec<u8>) -> anyhow::Result<()> {
        sync::transit(&codec::decode(&block)?, &self.db, &self.validator)
    }
}
