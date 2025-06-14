//! Account registry with cached state

use crate::Storage;
use anyhow::Result;
pub use registry::Accounts;
use score::{
    StorageKey,
    service::{GasLimit, ServiceAccount, ServiceAccountState},
};
use std::sync::Arc;

mod registry;

/// Account with cached state
pub struct Account<S: Storage> {
    /// The storage of the account
    storage: Arc<S>,

    /// The index of the account
    index: u32,

    /// The account state
    account: ServiceAccount,

    /// The removed keys of the account
    removed: Vec<StorageKey>,
}

impl<S: Storage> Account<S> {
    /// Create a new account
    pub fn new(storage: Arc<S>, index: u32) -> Result<Self> {
        let account = storage.account(index)?;
        Ok(Self {
            storage,
            index,
            account,
            removed: Vec::new(),
        })
    }
}

impl<S: Storage> score::account::Account for Account<S> {
    fn account(&self) -> ServiceAccount {
        self.account.clone()
    }

    fn balance(&self) -> u64 {
        self.account.balance
    }

    fn balance_mut(&mut self) -> &mut u64 {
        &mut self.account.balance
    }

    fn blob(&self) -> Option<Vec<u8>> {
        None
    }

    fn code(&self) -> [u8; 32] {
        self.account.code
    }

    fn set_code(&mut self, code: [u8; 32]) {
        self.account.code = code;
    }

    fn gas(&self) -> GasLimit {
        self.account.gas.clone()
    }

    fn set_gas(&mut self, gas: GasLimit) {
        self.account.gas = gas;
    }

    fn threshold(&self) -> u64 {
        self.account.threshold()
    }

    fn total(&self) -> u64 {
        self.account.balance
    }

    fn items(&self) -> u32 {
        self.account.items()
    }

    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Vec<u32>> {
        self.account.lookup.get(&(hash, len)).cloned()
    }

    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, lookup: Vec<u32>) {
        self.account.lookup.insert((hash, len), lookup);
    }

    fn remove_lookup(&mut self, hash: [u8; 32], len: u32) {
        self.account.lookup.remove(&(hash, len));
    }

    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>> {
        self.account.preimage.get(&hash).cloned()
    }

    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        self.account.preimage.insert(hash, preimage);
    }

    fn remove_preimage(&mut self, hash: [u8; 32]) {
        self.account.preimage.remove(&hash);
    }

    fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        self.account.storage.get(key)
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        self.account.storage.insert(key.to_vec(), value);
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.account.storage.remove(key)
    }

    fn info(&self) -> ServiceAccountState {
        ServiceAccountState {
            code: self.account.code,
            balance: self.account.balance,
            threshold: self.account.threshold(),
            transfer: self.account.gas.transfer,
            accumulate: self.account.gas.accumulate,
            total: self.account.balance,
            items: self.account.items(),
        }
    }
}

impl<S: Storage> Clone for Account<S> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            index: self.index,
            account: self.account.clone(),
            removed: self.removed.clone(),
        }
    }
}
