//! The validator service of SpaceJam

use score::{
    block::{history::BlockInfo, Block, Extrinsic, Header},
    state::{key, Storage},
};
pub use validate::Validation;

mod author;
mod validate;

/// The validator of SpaceJam
///
/// currently just an empty type which can calculate next blocks without any signatures.
#[derive(Default)]
pub struct Validator;

impl Validator {
    /// Mine the block
    pub fn mine(&self, block: BlockInfo, db: &impl Storage) -> anyhow::Result<Block> {
        let mut header = Header {
            parent: block.header_hash,
            parent_state_root: block.state_root,
            ..Default::default()
        };

        // TODO: handle the transaction pool.
        let extrinsic: Extrinsic = Default::default();
        header.extrinsic_hash = extrinsic.hash()?;
        header.slot = db.timeslot()?.unwrap_or(0) + 1;
        header.epoch_mark = None;
        header.tickets_mark = None;
        header.offenders_mark = vec![];
        header.author_index = 0;
        header.entropy_source = [0u8; 96];
        header.seal = [0u8; 96];

        // write the new state to the database
        db.set(key::TIMESLOT, header.slot.to_le_bytes())?;
        Ok(Block { header, extrinsic })
    }
}
