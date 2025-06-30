//! Account registry

use crate::{account::Account, service::ServiceAccount, OpaqueHash, TrieKey};
use std::collections::BTreeMap;

/// Account registry
pub trait Accounts: Clone {
    /// Get the code of an account
    fn code(&mut self, index: u32) -> Option<Vec<u8>>;

    /// Get the code hash of an account
    fn code_hash(&self, index: u32) -> Option<OpaqueHash>;

    /// Create a new account
    fn upsert(&mut self, index: u32, account: impl Account);

    /// Get the services from the registry
    fn services(&self) -> Vec<u32>;

    /// Get an account from the registry
    fn get(&mut self, index: u32) -> Option<&mut impl Account>;

    /// Remove an account from the registry
    fn remove(&mut self, index: u32);

    /// Batch all accounts from the registry
    fn accounts(&self) -> &BTreeMap<u32, impl Account>;

    /// Get the diff of the accounts
    fn diff(self) -> (Vec<(TrieKey, Vec<u8>)>, Vec<TrieKey>);
}

impl Accounts for BTreeMap<u32, ServiceAccount> {
    fn code(&mut self, index: u32) -> Option<Vec<u8>> {
        self.get(index)?.blob()
    }

    fn code_hash(&self, index: u32) -> Option<OpaqueHash> {
        Some(self.get(&index)?.code)
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

    fn accounts(&self) -> &BTreeMap<u32, impl Account> {
        self
    }

    fn diff(self) -> (Vec<(TrieKey, Vec<u8>)>, Vec<TrieKey>) {
        unimplemented!("account diff not implemented")
    }
}
