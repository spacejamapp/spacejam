//! Runtime utilities of SpaceJam

use crate::Block;
pub use {storage::Storage, validator::Validator};

pub mod storage;
pub mod tx;
mod validator;

/// Runtime of SpaceJam
pub struct Runtime<S: Storage, V: Validator> {
    /// The validator of SpaceJam
    pub validator: V,

    /// The storage of SpaceJam
    pub storage: S,
}

impl<S: Storage, V: Validator> Runtime<S, V> {
    /// Create a new runtime
    pub fn new(validator: V, storage: S) -> Self {
        Self { validator, storage }
    }

    /// Author a block
    pub async fn author(&self) -> anyhow::Result<Block> {
        let block = self.storage.recent_blocks()?;
        let Some(block) = block.and_then(|b| b.last().cloned()) else {
            anyhow::bail!("genesis block not found");
        };

        Block::builder()
            .parent(&block)?
            .seal(&self.validator, &self.storage)
    }

    /// Import a block
    pub fn import(&self, block: Vec<u8>) -> anyhow::Result<()> {
        tx::transit(&codec::decode(&block)?, &self.storage, &self.validator)
    }
}
