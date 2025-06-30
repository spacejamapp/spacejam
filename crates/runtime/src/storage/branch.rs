//! Branch of state

use crate::{
    storage::{Commit, KVStorage},
    Storage,
};
use anyhow::Result;
use score::TrieKey;
use std::{
    collections::{hash_map::IntoIter, HashMap},
    sync::{Arc, RwLock},
};

/// A branch of the state
pub struct Branch<S: Storage> {
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
}

impl<S: Storage> KVStorage for Branch<S> {
    fn commit(&self, commit: Commit<Vec<u8>, Vec<u8>>) -> Result<()> {
        for (key, value) in commit.iset() {
            self.set(key, value)?;
        }

        let mut diff = self
            .diff
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to aquire diff lock"))?;
        for key in commit.iremoval() {
            diff.remove(key);
        }

        Ok(())
    }

    fn commit_legacy(&self, commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        for (key, value) in commit.iset() {
            self.set(key, value)?;
        }

        let mut diff = self
            .diff
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to aquire diff lock"))?;
        for key in commit.iremoval() {
            diff.remove(key.as_ref());
        }

        Ok(())
    }

    fn set(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let mut diff = self
            .diff
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to aquire diff lock"))?;
        diff.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        Ok(())
    }

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let diff = self
            .diff
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to aquire diff lock"))?;

        if let Some(value) = diff.get(key.as_ref()) {
            return Ok(Some(value.clone()));
        }

        self.state.get(key)
    }

    fn iter(&self) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let diff = self
            .diff
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to aquire diff lock"))?;

        Ok(BranchIter {
            diff: diff.clone(),
            state: self.state.iter()?,
            finished: false,
            iter: Default::default(),
        })
    }

    fn prefix_iter(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let diff = self
            .diff
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to aquire diff lock"))?;

        Ok(BranchIter {
            diff: diff
                .iter()
                .filter(|(key, _)| key.starts_with(prefix.as_ref()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            state: self.state.iter()?,
            finished: false,
            iter: Default::default(),
        })
    }

    fn is_empty(&self) -> bool {
        let Ok(diff) = self.diff.read() else {
            return false;
        };

        diff.is_empty() && self.state.is_empty()
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
