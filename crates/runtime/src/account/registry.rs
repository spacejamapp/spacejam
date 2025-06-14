//! Account registry with cached state

use crate::{Storage, account::Account};
use score::{account::Account as _, service::ServiceAccount};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    sync::Arc,
};

/// Account registry with cached state
pub struct Accounts<S: Storage> {
    /// The storage of the accounts
    storage: Arc<S>,

    /// The account registry
    accounts: BTreeMap<u32, Account<S>>,

    /// The removed accounts
    removed: BTreeSet<u32>,
}

impl<S: Storage> Accounts<S> {
    /// Create a new account registry
    ///
    /// TODO: fetch initial accounts based on input extrinsics
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            storage,
            accounts: BTreeMap::new(),
            removed: BTreeSet::new(),
        }
    }

    /// Remove an account from the registry
    pub fn remove(&mut self, index: u32) {
        self.accounts.remove(&index);
        self.removed.insert(index);
    }
}

impl<S: Storage> score::account::Accounts for Accounts<S> {
    fn get(&mut self, index: u32) -> Option<&mut (impl score::account::Account + '_)> {
        if let Entry::Vacant(e) = self.accounts.entry(index) {
            e.insert(Account::new(self.storage.clone(), index).ok()?);
        }

        self.accounts.get_mut(&index)
    }

    fn upsert(&mut self, index: u32, account: ServiceAccount) {
        self.accounts.insert(
            index,
            Account {
                storage: self.storage.clone(),
                index,
                account,
                removed: Default::default(),
            },
        );
    }

    fn remove(&mut self, index: u32) {
        self.accounts.remove(&index);
    }

    fn code(&self, index: u32) -> Option<Vec<u8>> {
        self.accounts
            .get(&index)
            .and_then(|a| a.blob().map(|b| b.to_vec()))
    }

    fn services(&self) -> Vec<u32> {
        self.accounts.keys().cloned().collect()
    }

    fn accounts(self) -> BTreeMap<u32, ServiceAccount> {
        self.accounts
            .into_iter()
            .map(|(k, v)| (k, v.account))
            .collect()
    }

    fn diff(self) -> (Vec<([u8; 31], Vec<u8>)>, Vec<[u8; 31]>) {
        todo!()
    }
}

impl<S: Storage> Clone for Accounts<S> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            accounts: self.accounts.clone(),
            removed: self.removed.clone(),
        }
    }
}
