//! The commit of the storage

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::{BTreeMap, BTreeSet};

/// A commit of storage
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Commit<Key: Ord, Value> {
    /// The set of the commit
    pub update: BTreeMap<Key, Value>,

    /// The remove of the commit
    pub removal: BTreeSet<Key>,
}

impl<Key, Value> Commit<Key, Value>
where
    Key: Ord + std::fmt::Debug + Serialize + DeserializeOwned,
    Value: Serialize + DeserializeOwned,
{
    /// The length of the commit
    pub fn len(&self) -> usize {
        self.update.len() + self.removal.len()
    }

    /// Check if the commit is empty
    pub fn is_empty(&self) -> bool {
        self.update.is_empty() && self.removal.is_empty()
    }

    /// Set a key pair to the storage
    pub fn set(&mut self, key: Key, value: Value) {
        self.update.insert(key, value);
    }

    /// Remove a key pair from the storage
    pub fn remove(&mut self, key: Key) {
        self.removal.insert(key);
    }

    /// Extend the commit
    pub fn extend(&mut self, other: Self) {
        self.update.extend(other.update);
        self.removal.extend(other.removal);
    }

    /// Extend the commit with an iterator
    pub fn extend_iter(
        &mut self,
        updates: impl IntoIterator<Item = (Key, Value)>,
        removals: impl IntoIterator<Item = Key>,
    ) {
        self.update.extend(updates);
        self.removal.extend(removals);
    }

    /// Iterate over the set of the commit
    pub fn iset(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.update.iter()
    }

    /// Iterate over the updates of the commit
    pub fn updates(self) -> impl Iterator<Item = (Key, Value)> {
        self.update.into_iter()
    }

    /// Iterate over the removal of the commit
    pub fn iremoval(&self) -> impl Iterator<Item = &Key> {
        self.removal.iter()
    }

    /// Iterate over the operations of the commit
    pub fn ops(self) -> impl Iterator<Item = Operation<Key, Value>> {
        self.update
            .into_iter()
            .map(|(key, value)| Operation::Set(key, value))
            .chain(self.removal.into_iter().map(|key| Operation::Remove(key)))
    }
}

impl<Key: Ord + Copy, Value> Commit<Key, Value> {
    /// Sorted, deduplicated union of all keys this commit touches.
    pub fn dirty_keys(&self) -> Vec<Key> {
        let mut keys: Vec<Key> = self
            .update
            .keys()
            .copied()
            .chain(self.removal.iter().copied())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        keys
    }
}

impl<Key, Value> Commit<Key, Value>
where
    Key: Ord + Copy,
    Value: AsRef<[u8]>,
{
    /// Merge-walk `base` ∪ `self.update` in key order, skipping `self.removal`.
    pub fn merge_with<'a>(&'a self, base: &'a BTreeMap<Key, Value>) -> Vec<(Key, &'a [u8])> {
        let mut kvs: Vec<(Key, &[u8])> = Vec::with_capacity(base.len() + self.update.len());
        let mut base_iter = base.iter();
        let mut diff_iter = self.update.iter();
        let mut b = base_iter.next();
        let mut d = diff_iter.next();
        while let Some((key, value)) = match (b, d) {
            (Some((bk, _)), Some((dk, dv))) if dk <= bk => {
                if dk == bk {
                    b = base_iter.next();
                }
                d = diff_iter.next();
                Some((*dk, dv.as_ref()))
            }
            (Some((bk, bv)), _) => {
                b = base_iter.next();
                Some((*bk, bv.as_ref()))
            }
            (None, Some((dk, dv))) => {
                d = diff_iter.next();
                Some((*dk, dv.as_ref()))
            }
            (None, None) => None,
        } {
            if !self.removal.contains(&key) {
                kvs.push((key, value));
            }
        }
        kvs
    }
}

impl<K, U, V, R> From<(U, R)> for Commit<K, V>
where
    U: IntoIterator<Item = (K, V)>,
    R: IntoIterator<Item = K>,
    K: Ord + Clone + Default,
    V: Clone + Default,
{
    fn from((updates, removals): (U, R)) -> Self {
        let mut commit = Commit::default();
        commit.update.extend(updates);
        commit.removal.extend(removals);
        commit
    }
}

/// An operation of a commit
#[derive(Debug, Clone)]
pub enum Operation<Key, Value> {
    /// Set a key pair to the storage
    Set(Key, Value),

    /// Remove a key pair from the storage
    Remove(Key),
}
