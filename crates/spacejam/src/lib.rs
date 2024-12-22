//! The runtime of SpaceJam
use score::{
    block::{history::BlockInfo, Block, BlocksHistory},
    state::Storage,
};
use validator::Validator;

pub mod cmd;
pub mod storage;
pub mod validator;

/// The runtime of SpaceJam
pub struct SpaceJam<Db: Storage> {
    /// The blocks history of the SpaceJam
    pub history: BlocksHistory,

    /// The database of SpaceJam
    pub db: Db,
}

impl<Db: Storage> SpaceJam<Db> {
    /// Initialize the chain with the given database.
    pub fn new(db: Db) -> Self {
        Self {
            history: BlocksHistory::default(),
            db,
        }
    }

    /// Mine a new block
    pub fn mine(&mut self) -> anyhow::Result<Block> {
        let validator = Validator::default();
        let last_block = if let Some(last_block) = self.history.blocks.last() {
            last_block.clone()
        } else {
            BlockInfo::default()
        };

        let block = validator.mine(last_block.clone(), &self.db)?;
        self.history.import(
            block.hash()?,
            block.header.parent_state_root,
            Default::default(),
            Default::default(),
        );
        Ok(block)
    }
}
