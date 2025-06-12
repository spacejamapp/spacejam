//! The parity database storage

#![cfg(feature = "parity")]

use anyhow::Result;
use parity_db::{BTreeIterator, ColumnOptions, Db, Operation as Op, Options};
use runtime::storage::{Commit, KVStorage, Operation};
use score::StorageKey;
use std::path::PathBuf;

/// The parity database storage
pub struct Parity(Db);

impl KVStorage for Parity {
    fn commit(&self, commit: Commit<StorageKey, Vec<u8>>) -> Result<()> {
        self.0
            .commit_changes(commit.operations().map(|op| match op {
                Operation::Set(k, v) => (0, Op::Set(k.to_vec(), v)),
                Operation::Remove(k) => (0, Op::Dereference(k.to_vec())),
            }))?;
        Ok(())
    }

    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.0
            .commit(vec![(0, key.as_ref(), Some(value.as_ref().to_vec()))])?;
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.0.get(0, key.as_ref())?)
    }

    fn iter(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        Ok(ParityIter(self.0.iter(0)?))
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        Ok(ParityPrefixIter {
            prefix: prefix.as_ref().to_vec(),
            inner: self.0.iter(0)?,
        })
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

/// The prefix iterator wrapper
pub struct ParityPrefixIter<'a> {
    prefix: Vec<u8>,
    inner: BTreeIterator<'a>,
}

impl Iterator for ParityPrefixIter<'_> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next().map_err(Into::into).transpose()?;

        match next {
            Ok((k, v)) => {
                if k.starts_with(&self.prefix) {
                    Some(Ok((k, v)))
                } else {
                    self.next()
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl TryFrom<PathBuf> for Parity {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let options = Options {
            path,
            columns: vec![ColumnOptions::default()],
            sync_wal: true,
            sync_data: true,
            stats: true,
            salt: None,
            compression_threshold: Default::default(),
        };
        Ok(Parity(Db::open(&options)?))
    }
}
