//! Account registry

use crate::{account::Account, service::ServiceAccount, StorageKey};
use std::collections::BTreeMap;

/// Account registry
pub trait Accounts: Clone {
    /// Get the code of an account
    fn code(&self, index: u32) -> Option<Vec<u8>>;

    /// Create a new account
    fn upsert(&mut self, index: u32, account: impl Account);

    /// Get the services from the registry
    fn services(&self) -> Vec<u32>;

    /// Get an account from the registry
    fn get(&mut self, index: u32) -> Option<&mut impl Account>;

    /// Remove an account from the registry
    fn remove(&mut self, index: u32);

    /// Batch all accounts from the registry
    fn accounts(self) -> BTreeMap<u32, ServiceAccount>;

    /// Get the diff of the accounts
    fn diff(self) -> (Vec<(StorageKey, Vec<u8>)>, Vec<StorageKey>);
}

impl Accounts for BTreeMap<u32, ServiceAccount> {
    fn code(&self, index: u32) -> Option<Vec<u8>> {
        self.get(&index)?.blob()
    }

    fn upsert(&mut self, index: u32, account: impl Account) {
        self.insert(index, account.account());
    }

    fn services(&self) -> Vec<u32> {
        self.keys().cloned().collect()
    }

    fn get(&mut self, index: u32) -> Option<&mut impl Account> {
        self.get_mut(&index)
    }

    fn remove(&mut self, index: u32) {
        self.remove(&index);
    }

    fn accounts(self) -> BTreeMap<u32, ServiceAccount> {
        self
    }

    fn diff(self) -> (Vec<(StorageKey, Vec<u8>)>, Vec<StorageKey>) {
        unimplemented!("account diff not implemented")
    }
}
