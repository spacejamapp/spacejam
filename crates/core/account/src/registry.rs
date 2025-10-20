//! Account registry

use crate::Account;
use score::{OpaqueHash, ServiceId, TrieKey, service::ServiceAccount};
use std::collections::{BTreeMap, BTreeSet};

/// Account registry
pub trait Accounts: Clone + Send + Sync + 'static {
    /// Get the code of an account
    fn blob(&mut self, index: u32) -> Option<Vec<u8>>;

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

    /// Get the removed accounts from the registry
    fn removed(&self) -> BTreeSet<u32> {
        Default::default()
    }

    /// Check if an account is removed
    fn is_removed(&self, index: u32) -> bool;

    /// Get the diff of the accounts
    fn diff(self) -> (Vec<(TrieKey, Vec<u8>)>, Vec<TrieKey>);

    /// Check and find a free account index
    fn check(&mut self, mut index: ServiceId) -> ServiceId {
        loop {
            if self.get(index).is_none() {
                return index;
            }

            index = ((index - (1 << 8) + 1) % score::CHECK_SALT) + (1 << 8);
        }
    }
}

impl Accounts for BTreeMap<u32, ServiceAccount> {
    fn blob(&mut self, index: u32) -> Option<Vec<u8>> {
        self.get(index)?.blob()
    }

    fn code_hash(&self, index: u32) -> Option<OpaqueHash> {
        Some(self.get(&index)?.info.code)
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
        BTreeMap::remove(self, &index);
    }

    fn is_removed(&self, index: u32) -> bool {
        !self.contains_key(&index)
    }

    fn accounts(&self) -> &BTreeMap<u32, impl Account> {
        self
    }

    fn diff(self) -> (Vec<(TrieKey, Vec<u8>)>, Vec<TrieKey>) {
        unimplemented!("account diff not implemented")
    }
}
