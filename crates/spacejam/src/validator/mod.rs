//! The validator service of SpaceJam

use score::{
    block::{Block, BlockInfo},
    state::{key, Storage},
    validator,
};
pub use validate::Validation;

mod author;
mod validate;

/// The validator of SpaceJam
///
/// currently just an empty type which can calculate next blocks without any signatures.
#[allow(unused)]
pub struct Validator<V: validator::Validator> {
    /// The inner validator
    inner: V,
}

impl<V: validator::Validator> Validator<V> {
    /// Creates a new validator
    pub fn new(inner: V) -> Self {
        Self { inner }
    }

    /// Mine the block
    pub fn mine(&self, block: BlockInfo, db: &impl Storage) -> anyhow::Result<Block> {
        let mut block = block.mine();

        // TODO: handle the transaction pool.
        block.header.extrinsic_hash = block.extrinsic.hash()?;
        block.header.slot = db.timeslot()?.unwrap_or(0) + 1;
        block.header.epoch_mark = None;
        block.header.tickets_mark = None;
        block.header.offenders_mark = vec![];
        block.header.author_index = 0;
        block.header.entropy_source = [0u8; 96];
        block.header.seal = [0u8; 96];

        // write the new state to the database
        db.set(key::TIMESLOT, block.header.slot.to_le_bytes())?;
        Ok(block)
    }
}

impl<V: validator::Validator> From<V> for Validator<V> {
    fn from(inner: V) -> Self {
        Self { inner }
    }
}
