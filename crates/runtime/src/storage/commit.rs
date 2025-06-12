//! The commit of the storage

use std::collections::{BTreeMap, BTreeSet};

/// A commit of storage
#[derive(Debug, Default, Clone)]
pub struct Commit<Key, Value> {
    /// The set of the commit
    set: BTreeMap<Key, Value>,

    /// The remove of the commit
    removal: BTreeSet<Key>,
}

impl<Key, Value> Commit<Key, Value>
where
    Key: Ord,
{
    /// Set a key pair to the storage
    pub fn set(&mut self, key: Key, value: Value) {
        self.set.insert(key, value);
    }

    /// Remove a key pair from the storage
    pub fn remove(&mut self, key: Key) {
        self.removal.insert(key);
    }

    /// Extend the commit
    pub fn extend(&mut self, other: Self) {
        self.set.extend(other.set);
        self.removal.extend(other.removal);
    }

    /// Iterate over the set of the commit
    pub fn iset(self) -> impl Iterator<Item = (Key, Value)> {
        self.set.into_iter()
    }

    /// Iterate over the removal of the commit
    pub fn iremoval(&self) -> impl Iterator<Item = &Key> {
        self.removal.iter()
    }

    /// Iterate over the operations of the commit
    pub fn operations(self) -> impl Iterator<Item = Operation<Key, Value>> {
        self.set
            .into_iter()
            .map(|(key, value)| Operation::Set(key, value))
            .chain(self.removal.into_iter().map(|key| Operation::Remove(key)))
    }
}

impl<T, K, V> From<T> for Commit<K, V>
where
    T: IntoIterator<Item = (K, V)>,
    K: Ord + Clone + Default,
    V: Clone + Default,
{
    fn from(kvs: T) -> Self {
        let mut commit = Commit::default();
        commit.set.extend(kvs);
        commit
    }
}

/// An operation of a commit
pub enum Operation<Key, Value> {
    /// Set a key pair to the storage
    Set(Key, Value),

    /// Remove a key pair from the storage
    Remove(Key),
}
