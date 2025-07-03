//! Branch of state

use crate::{
    storage::{Column, Commit, KVStorage, StateStorage},
    Storage,
};
use anyhow::Result;
use score::TrieKey;
use std::{
    collections::{hash_map::IntoIter, HashMap},
    sync::{Arc, RwLock},
};

/// A branch of the state
pub struct Branch<S: StateStorage> {
    /// The state of the branch
    state: Arc<S>,

    /// The diff of the branch
    diff: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl<S: Storage> Branch<S> {
    /// Create a new branch from a state
    pub fn checkout(state: Arc<S>) -> Self {
        Self {
            state,
            diff: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the state of the branch
    pub fn state(&self) -> Arc<S> {
        self.state.clone()
    }
}

impl<S: Storage> KVStorage for Branch<S> {
    fn commit(&self, _column: Column, commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        let mut diff = self
            .diff
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire diff lock"))?;

        for (key, value) in commit.iset() {
            diff.insert(key.to_vec(), value.clone());
        }

        for key in commit.iremoval() {
            diff.remove(key.as_ref());
        }

        Ok(())
    }

    fn set(&self, _column: Column, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let mut diff = self
            .diff
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire diff lock"))?;
        diff.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        Ok(())
    }

    fn get(&self, _column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let diff = self
            .diff
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire diff lock"))?;

        if let Some(value) = diff.get(key.as_ref()) {
            return Ok(Some(value.clone()));
        }

        self.state.state_get(key)
    }

    fn iter(&self, _column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let diff = self
            .diff
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire diff lock"))?;

        Ok(BranchIter {
            diff: diff.clone(),
            state: self.state.state_iter()?,
            finished: false,
            iter: Default::default(),
        })
    }

    fn prefix_iter(
        &self,
        _column: Column,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let diff = self
            .diff
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire diff lock"))?;

        Ok(BranchIter {
            diff: diff
                .iter()
                .filter(|(key, _)| key.starts_with(prefix.as_ref()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            state: self.state.state_iter()?,
            finished: false,
            iter: Default::default(),
        })
    }
}

impl<S: Storage> Clone for Branch<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            diff: self.diff.clone(),
        }
    }
}

/// Iterator over the branch
pub struct BranchIter<I: Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
    /// The diff of the branch
    diff: HashMap<Vec<u8>, Vec<u8>>,

    /// The state of the branch
    state: I,

    /// The iterator over the diff
    iter: IntoIter<Vec<u8>, Vec<u8>>,

    /// If the state iterator is finished
    finished: bool,
}

impl<I: Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> Iterator for BranchIter<I> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return self.iter.next().map(Ok);
        }

        // If the state iterator is finished, we need to return the next diff entry
        let Some(next) = self.state.next() else {
            self.finished = true;
            self.iter = self.diff.clone().into_iter();
            self.diff.clear();
            return self.next();
        };

        // if the next value is an error, return it
        let Ok((key, value)) = next else {
            return Some(next);
        };

        // override the next value with the diff value if exists
        let Some((key, value)) = self
            .diff
            .get_key_value(&key)
            .map(|(key, value)| (key.clone(), value.clone()))
        else {
            return Some(Ok((key, value)));
        };

        self.diff.remove(&key);
        Some(Ok((key, value)))
    }
}
