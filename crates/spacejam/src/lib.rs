//! The runtime of SpaceJam
use score::block::{Block, BlockInfo, BlocksHistory};
pub use score::{validator::Validator, Config};

pub mod cmd;
pub mod storage;
pub mod validator;

/// The runtime of SpaceJam
///
/// NOTE: need to get avoid of too many generic parameters...
pub struct SpaceJam<C: Config> {
    /// The blocks history of the SpaceJam
    pub history: BlocksHistory,

    /// The validator of SpaceJam
    pub validator: C::Validator,

    /// The storage of SpaceJam
    pub db: C::Db,
}

impl<C: Config> SpaceJam<C> {
    /// Initialize the chain with the given database.
    pub fn new(db: C::Db, validator: C::Validator) -> Self {
        Self {
            history: BlocksHistory::default(),
            validator,
            db,
        }
    }

    /// Mine a new block
    pub fn mine(&mut self) -> anyhow::Result<Block> {
        let last_block = if let Some(last_block) = self.history.blocks.last() {
            last_block.clone()
        } else {
            BlockInfo::default()
        };

        let block = self.validator.mine(&last_block, &self.db)?;
        self.history.import(
            block.hash()?,
            block.header.parent_state_root,
            Default::default(),
            Default::default(),
        );
        Ok(block)
    }
}
