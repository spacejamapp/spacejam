//! The parity database storage

use anyhow::Result;
use parity_db::{BTreeIterator, ColumnOptions, Db, Operation as Op, Options};
use runtime::storage::{Commit, KVStorage, Operation};
use std::path::PathBuf;

/// The column for the state
const STATE_COLUMN: u8 = 0;

/// The column for the sync
const SYNC_COLUMN: u8 = 1;

/// The parity database storage
pub struct Parity(Db);

impl KVStorage for Parity {
    fn commit(&self, commit: Commit<Vec<u8>, Vec<u8>>) -> Result<()> {
        self.0.commit_changes(commit.ops().map(|op| match op {
            Operation::Set(k, v) => (STATE_COLUMN, Op::Set(k.to_vec(), v)),
            Operation::Remove(k) => (STATE_COLUMN, Op::Dereference(k.to_vec())),
        }))?;
        Ok(())
    }

    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.0.commit(vec![(
            STATE_COLUMN,
            key.as_ref(),
            Some(value.as_ref().to_vec()),
        )])?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.0.get(STATE_COLUMN, key.as_ref())?)
    }

    fn iter(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        Ok(ParityIter(self.0.iter(STATE_COLUMN)?))
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let mut iter = self.0.iter(STATE_COLUMN)?;
        iter.seek(prefix.as_ref())?;
        Ok(ParityIter(iter))
    }
}

/// The iterator wrapper
pub struct ParityIter<'a>(BTreeIterator<'a>);

impl Iterator for ParityIter<'_> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map_err(Into::into).transpose()
    }
}

impl TryFrom<PathBuf> for Parity {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let options = Options {
            path,
            columns: vec![ColumnOptions {
                btree_index: true,
                ..Default::default()
            }],
            sync_wal: true,
            sync_data: true,
            stats: true,
            salt: None,
            compression_threshold: Default::default(),
        };
        Ok(Parity(Db::open_or_create(&options)?))
    }
}
