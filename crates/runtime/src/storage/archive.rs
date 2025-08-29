//! Archived storage

use std::sync::Arc;

use crate::storage::{Column, Commit, KVStorage};
use anyhow::Result;
use score::{state::StateKeyLike, OpaqueHash, TrieKey};

/// The archived storage
pub trait ArchiveStorage: KVStorage + Send + Sync + 'static {
    /// Archive a block
    fn archive(&self, block: &OpaqueHash) -> Result<()> {
        let mut commit = Commit::default();
        let iter = self.iter(Column::State)?;
        for pair in iter {
            let (key, value) = pair?;
            let key = [block[..6].to_vec(), key].concat().as_state_key();
            commit.set(key, value);
        }

        self.commit(Column::Archive, commit)?;
        Ok(())
    }
}

/// The archived storage
pub struct Archive<S: KVStorage> {
    /// The block that is archived
    block: OpaqueHash,

    /// The state of the archive
    state: Arc<S>,
}

impl<S: KVStorage> Archive<S> {
    /// Create a new archive
    pub fn checkout(state: Arc<S>, block: OpaqueHash) -> Self {
        Self { block, state }
    }
}

impl<S: KVStorage> KVStorage for Archive<S> {
    fn commit(&self, _column: Column, _commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        anyhow::bail!("commit is not allowed on archive")
    }

    fn set(&self, _column: Column, _key: impl AsRef<[u8]>, _value: impl AsRef<[u8]>) -> Result<()> {
        anyhow::bail!("set is not allowed on archive")
    }

    fn get(&self, _column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let key = [self.block[..6].to_vec(), key.as_ref().to_vec()]
            .concat()
            .as_state_key();
        self.state.get(Column::Archive, key)
    }

    fn iter(&self, _column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        self.state
            .prefix_iter(Column::Archive, self.block[..6].to_vec())
    }

    fn prefix_iter(
        &self,
        _column: Column,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let prefix = [self.block[..6].to_vec(), prefix.as_ref().to_vec()].concat();
        self.state.prefix_iter(Column::Archive, prefix)
    }
}
