//! The commit of the storage

use std::collections::{BTreeMap, BTreeSet};

/// A commit of storage
#[derive(Debug, Default, Clone)]
pub struct Commit<Key, Value> {
    /// The set of the commit
    update: BTreeMap<Key, Value>,

    /// The remove of the commit
    removal: BTreeSet<Key>,
}

impl<Key, Value> Commit<Key, Value>
where
    Key: Ord,
{
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
