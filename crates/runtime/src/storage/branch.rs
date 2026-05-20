//! Branch of state

use crate::{
    storage::{Column, Commit, KVStorage, StateStorage},
    Storage,
};
use anyhow::Result;
use score::{state::StateKeyLike, TrieKey};
use std::{
    collections::{btree_map::IntoIter, BTreeMap},
    mem,
    sync::{Arc, RwLock},
};

/// A branch of the state
pub struct Branch<S: StateStorage> {
    /// The state of the branch
    state: Arc<S>,

    /// The accumulated commit of the branch
    pub commit: Arc<RwLock<Commit<TrieKey, Vec<u8>>>>,
}

impl<S: Storage> Branch<S> {
    /// Create a new branch from a state
    pub fn checkout(state: Arc<S>) -> Self {
        Self {
            state,
            commit: Arc::new(RwLock::new(Commit::default())),
        }
    }

    /// Get the state of the branch
    pub fn state(&self) -> Arc<S> {
        self.state.clone()
    }
}

impl<S: Storage> KVStorage for Branch<S> {
    fn commit(&self, _column: Column, new_commit: Commit<TrieKey, Vec<u8>>) -> Result<()> {
        let mut commit = self
            .commit
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire commit lock"))?;

        // Merge the new commit with the existing one
        commit.extend(new_commit);
        Ok(())
    }

    fn set(&self, _column: Column, _key: impl AsRef<[u8]>, _value: impl AsRef<[u8]>) -> Result<()> {
        anyhow::bail!("set is not allowed on branch")
    }

    fn get(&self, _column: Column, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        let commit = self
            .commit
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire commit lock"))?;

        let trie_key = key.as_ref().as_state_key();

        // Check if the key is marked for removal (removals take precedence)
        if commit.removal.contains(&trie_key) {
            return Ok(None);
        }

        // Check if the key exists in the updates
        if let Some(value) = commit.update.get(&trie_key) {
            return Ok(Some(value.clone()));
        }

        // Fall back to the underlying state
        self.state.state_get(key)
    }

    fn iter(&self, _column: Column) -> Result<impl Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
        let commit = self
            .commit
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire commit lock"))?;

        Ok(BranchIter {
            updates: commit.update.clone(),
            removals: commit.removal.clone(),
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
        let commit = self
            .commit
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire commit lock"))?;

        let prefix_bytes = prefix.as_ref();
        let filtered_updates: BTreeMap<TrieKey, Vec<u8>> = commit
            .update
            .iter()
            .filter(|(key, _)| key.as_ref().starts_with(prefix_bytes))
            .map(|(key, value)| (*key, value.clone()))
            .collect();

        Ok(BranchIter {
            updates: filtered_updates,
            removals: commit.removal.clone(),
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
            commit: self.commit.clone(),
        }
    }
}

/// Iterator over the branch
pub struct BranchIter<I: Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> {
    /// The updates from the commit
    updates: BTreeMap<TrieKey, Vec<u8>>,

    /// The removals from the commit
    removals: std::collections::BTreeSet<TrieKey>,

    /// The state iterator
    state: I,

    /// The iterator over the updates
    iter: IntoIter<TrieKey, Vec<u8>>,

    /// If the state iterator is finished
    finished: bool,
}

impl<I: Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> Iterator for BranchIter<I> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return loop {
                let (k, v) = self.iter.next()?;
                if !self.removals.contains(&k) {
                    break Some(Ok((k.to_vec(), v)));
                }
            };
        }

        // If the state iterator is finished, we need to return the next update entry
        let Some(next) = self.state.next() else {
            self.finished = true;
            self.iter = mem::take(&mut self.updates).into_iter();
            return self.next();
        };

        // if the next value is an error, return it
        let Ok((key, value)) = next else {
            return Some(next);
        };

        let trie_key = key.as_state_key();

        // Skip if the key is marked for removal
        if self.removals.contains(&trie_key) {
            self.updates.remove(&trie_key);
            return self.next();
        }

        // Override with the update value if it exists
        if let Some(updated) = self.updates.remove(&trie_key) {
            return Some(Ok((key, updated)));
        }

        // Return the state value if not in updates and not removed
        Some(Ok((key, value)))
    }
}
