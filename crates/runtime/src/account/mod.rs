//! Account registry with cached state

use crate::Storage;
pub use registry::Accounts;
use score::{
    StorageKey,
    service::{GasLimit, ServiceAccount, ServiceAccountState},
};
use std::collections::btree_map::Entry;

mod registry;

/// Account with cached state
#[derive(Clone)]
pub struct Account<'a, S: Storage> {
    /// The storage of the account
    storage: &'a S,

    /// The index of the account
    index: u32,

    /// The account state
    account: ServiceAccount,

    /// The removed keys of the account
    removed: Vec<StorageKey>,
}

impl<'a, S: Storage> Account<'a, S> {
    /// Create a new account
    pub fn new(storage: &'a S, index: u32, state: ServiceAccountState) -> Self {
        Self {
            storage,
            index,
            account: ServiceAccount {
                code: state.code,
                balance: state.balance,
                gas: GasLimit {
                    transfer: state.transfer,
                    accumulate: state.accumulate,
                },
                ..Default::default()
            },
            removed: Vec::new(),
        }
    }

    /// Insert a preimage to the account
    pub fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        self.account.preimage.insert(hash, preimage);
    }

    /// Get a lookup from the account
    pub fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<&mut Vec<u32>> {
        if let Entry::Vacant(e) = self.account.lookup.entry((hash, len)) {
            let lookup = self.storage.account_lookup(self.index, len, hash).ok()?;
            e.insert(lookup);
        }

        self.account.lookup.get_mut(&(hash, len))
    }

    /// Get a preimage from the account
    pub fn preimage(&mut self, hash: [u8; 32]) -> Option<&Vec<u8>> {
        if let Entry::Vacant(e) = self.account.preimage.entry(hash) {
            let preimage = self.storage.account_preimage(self.index, hash).ok()?;
            e.insert(preimage);
        }

        self.account.preimage.get(&hash)
    }

    /// Upsert a lookup to the account
    pub fn upsert_lookup(&mut self, hash: [u8; 32], len: u32, lookup: Vec<u32>) {
        self.account.lookup.insert((hash, len), lookup);
    }

    /// Read a value from the account
    pub fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        if !self.account.storage.contains_key(key) {
            let value = self.storage.account_storage(self.index, key).ok()?;
            self.account.storage.insert(key.to_vec(), value.clone());
        }

        self.account.storage.get(key)
    }

    /// Write a value to the account
    pub fn write(&mut self, key: &[u8], value: Vec<u8>) {
        self.account.storage.insert(key.to_vec(), value);
    }

    /// Remove a key from the account
    pub fn remove(&mut self, key: [u8; 31]) {
        self.account.storage.remove(key.as_slice());
        self.removed.push(key);
    }
}
