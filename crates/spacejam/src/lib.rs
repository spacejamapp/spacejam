//! The runtime of SpaceJam
use score::block::{history::BlockInfo, Block, BlocksHistory};
pub use {config::Config, validator::Validator};

pub mod cmd;
mod config;
pub mod storage;
pub mod validator;

/// The runtime of SpaceJam
///
/// NOTE: need to get avoid of too many generic parameters...
pub struct SpaceJam<C: Config> {
    /// The blocks history of the SpaceJam
    pub history: BlocksHistory,

    /// The database of SpaceJam
    pub db: C::Db,

    /// The validator of SpaceJam
    pub validator: Validator<C::Validator>,
}

impl<C: Config> SpaceJam<C> {
    /// Initialize the chain with the given database.
    pub fn new(db: C::Db, validator: Validator<C::Validator>) -> Self {
        Self {
            history: BlocksHistory::default(),
            db,
            validator,
        }
    }

    /// Mine a new block
    pub fn mine(&mut self) -> anyhow::Result<Block> {
        let last_block = if let Some(last_block) = self.history.blocks.last() {
            last_block.clone()
        } else {
            BlockInfo::default()
        };

        let block = self.validator.mine(last_block.clone(), &self.db)?;
        self.history.import(
            block.hash()?,
            block.header.parent_state_root,
            Default::default(),
            Default::default(),
        );
        Ok(block)
    }
}
