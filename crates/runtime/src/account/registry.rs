//! Account registry with cached state

use crate::{Storage, account::Account};
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

/// Account registry with cached state
#[derive(Clone)]
pub struct Accounts<'a, S: Storage> {
    /// The storage of the accounts
    storage: &'a S,

    /// The account registry
    accounts: BTreeMap<u32, Account<'a, S>>,

    /// The removed accounts
    removed: BTreeSet<u32>,
}

impl<'a, S: Storage> Accounts<'a, S> {
    /// Create a new account registry
    ///
    /// TODO: fetch initial accounts based on input extrinsics
    pub fn new(storage: &'a S) -> Self {
        Self {
            storage,
            accounts: BTreeMap::new(),
            removed: BTreeSet::new(),
        }
    }

    /// Get an account from the registry
    pub fn get(&mut self, index: u32) -> Option<&mut Account<'a, S>> {
        if let Entry::Vacant(e) = self.accounts.entry(index) {
            e.insert(Account::new(self.storage, index).ok()?);
        }

        self.accounts.get_mut(&index)
    }

    /// Remove an account from the registry
    pub fn remove(&mut self, index: u32) {
        self.accounts.remove(&index);
        self.removed.insert(index);
    }
}
