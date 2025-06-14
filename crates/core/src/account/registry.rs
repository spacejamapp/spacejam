//! Account registry

use crate::{account::Account, service::ServiceAccount, StorageKey};
use std::collections::BTreeMap;

/// Account registry
pub trait Accounts {
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
