//! Context for SpaceJam

use score::{state::Storage, validator::Validator};

/// The context for SpaceJam
pub struct Context<S: Storage, V: Validator> {
    /// The validator of SpaceJam
    pub validator: V,

    /// The storage of SpaceJam
    pub db: S,
}

impl<S: Storage, V: Validator> Context<S, V> {
    /// Create a new context
    pub fn new(validator: V, db: S) -> Self {
        Self { validator, db }
    }
}

impl<S: Storage, V: Validator> network::Context for Context<S, V> {
    fn import_block(&self, block: Vec<u8>) -> anyhow::Result<()> {
        sync::transit(&codec::decode(&block)?, &self.db, &self.validator)
    }
}
