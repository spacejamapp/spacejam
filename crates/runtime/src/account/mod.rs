//! Account registry with cached state

use crate::{storage::Commit, Storage};
use anyhow::Result;
use pvm::Gas;
pub use registry::Accounts;
use score::{
    service::{ServiceAccount, ServiceInfo},
    state::account,
    TrieKey,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

mod registry;

/// Account with cached state
pub struct Account<S: Storage> {
    /// The storage of the account
    state: Arc<S>,

    /// The index of the account
    index: u32,

    /// The account state
    account: ServiceAccount,

    /// The operations of the account
    ops: Commit<TrieKey, Vec<u8>>,
}

impl<S: Storage> Account<S> {
    /// Create a new account
    pub fn new(storage: Arc<S>, index: u32) -> Result<Self> {
        let account = storage.account(index)?;

        Ok(Self {
            state: storage,
            index,
            account,
            ops: Commit::default(),
        })
    }

    /// Read storage via storage key
    pub fn hread(&self, key: TrieKey) -> Option<Vec<u8>> {
        if self.account.storage.contains_key(key.as_slice()) {
            self.account.storage.get(key.as_slice()).cloned()
        } else {
            self.state.state_get(&key).ok().flatten()
        }
    }

    /// Inherit from another account
    pub fn inherit(storage: Arc<S>, index: u32, account: impl score::Account) -> Self {
        Self {
            state: storage,
            index,
            account: account.account(),
            ops: account.ops().into(),
        }
    }

    /// Drop a lookup if it exists
    pub fn drop_lookup(&mut self, hash: [u8; 32], len: u32) -> TrieKey {
        let key = account::lookup(self.index, len, hash);
        let mut mhash = [0; 32];
        mhash[..31].copy_from_slice(&key);
        self.account.lookup.remove(&(mhash, len));
        key
    }
}

impl<S: Storage> score::Account for Account<S> {
    fn index(&self) -> u32 {
        self.index
    }

    fn account(&self) -> ServiceAccount {
        self.account.clone()
    }

    fn balance(&self) -> u64 {
        self.account.info.balance
    }

    fn balance_mut(&mut self) -> &mut u64 {
        &mut self.account.info.balance
    }

    fn blob(&self) -> Option<Vec<u8>> {
        self.account.blob()
    }

    fn code(&self) -> [u8; 32] {
        self.account.info.code
    }

    fn set_code(&mut self, code: [u8; 32]) {
        self.account.info.code = code;
    }

    fn accumulate_gas(&self) -> Gas {
        self.account.info.accumulate
    }

    fn set_accumulate_gas(&mut self, gas: Gas) {
        self.account.info.accumulate = gas;
    }

    fn transfer_gas(&self) -> Gas {
        self.account.info.transfer
    }

    fn set_transfer_gas(&mut self, gas: Gas) {
        self.account.info.transfer = gas;
    }

    fn threshold(&self) -> u64 {
        self.account.threshold()
    }

    fn total(&self) -> u64 {
        self.account.info.total
    }

    fn set_total(&mut self, total: u64) {
        self.account.set_total(total);
    }

    fn items(&self) -> u32 {
        self.account.items()
    }

    fn set_items(&mut self, items: u32) {
        self.account.set_items(items);
    }

    fn creation(&self) -> u32 {
        self.account.info.creation
    }

    fn set_creation(&mut self, creation: u32) {
        self.account.info.creation = creation;
    }

    fn update(&self) -> u32 {
        self.account.info.update
    }

    fn set_update(&mut self, update: u32) {
        self.account.info.update = update;
    }

    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Vec<u32>> {
        if let Some(lookup) = self.account.lookup.get(&(hash, len)) {
            return Some(lookup.clone());
        }

        if let Ok(lookup) = self.state.account_lookup(self.index, len, hash) {
            self.drop_lookup(hash, len);
            self.account.lookup.insert((hash, len), lookup.clone());
            Some(lookup.clone())
        } else {
            None
        }
    }

    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, lookup: Vec<u32>) {
        let key = self.drop_lookup(hash, len);
        self.account.lookup.insert((hash, len), lookup.clone());
        self.ops.removal.remove(&key);
        self.ops
            .set(key, codec::encode(&lookup).expect("lookup is valid"));
        self.set_total(self.total() + 81 + len as u64);
        self.set_items(self.items() + 2);
    }

    fn remove_lookup(&mut self, hash: [u8; 32], len: u32) {
        let key = self.drop_lookup(hash, len);
        self.account.lookup.remove(&(hash, len));
        self.ops.remove(key);
        self.set_total(self.total() - 81 - len as u64);
        self.set_items(self.items() - 2);
    }

    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>> {
        self.account.preimage.get(&hash).cloned()
    }

    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        self.account.preimage.insert(hash, preimage.clone());
        self.ops.set(account::preimage(self.index, hash), preimage);
    }

    fn remove_preimage(&mut self, hash: [u8; 32]) {
        self.account.preimage.remove(&hash);
        let key = account::preimage(self.index, hash);
        self.ops.update.remove(&key);
        self.ops.removal.insert(key);
    }

    fn read(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let key = account::storage(self.index, key);
        self.hread(key)
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        let vkey = account::storage(self.index, key);

        // update total
        if let Some(old) = self.hread(vkey) {
            self.set_total(self.total() + value.len() as u64 - old.len() as u64);
        } else {
            self.set_items(self.items() + 1);
            self.set_total(self.total() + 34 + key.len() as u64 + value.len() as u64);
        }

        // update storage
        self.ops.removal.remove(&vkey);
        self.ops.set(vkey, value.clone());
        self.account.storage.insert(vkey.to_vec(), value);
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let vkey = account::storage(self.index, key);
        if self.ops.removal.contains(&vkey) {
            return None;
        }

        // update total
        let mut removed = None;
        if let Some(old) = self.hread(vkey) {
            self.set_total(self.total() - 34 - key.len() as u64 - old.len() as u64);
            self.set_items(self.items() - 1);
            self.ops.removal.insert(vkey);
            removed = Some(old);
        }

        self.account.storage.remove(vkey.as_slice());
        removed
    }

    fn info(&self) -> ServiceInfo {
        self.account.info.clone()
    }

    fn ops(mut self) -> (BTreeMap<TrieKey, Vec<u8>>, BTreeSet<TrieKey>) {
        self.ops.set(
            account::info(self.index),
            codec::encode(&self.account.state()).expect("data is valid"),
        );

        let removals: BTreeSet<TrieKey> = self.ops.iremoval().cloned().collect();
        let updates: BTreeMap<TrieKey, Vec<u8>> =
            self.ops.updates().map(|(k, v)| (k, v.clone())).collect();

        tracing::debug!("removals: {:?}", removals.len());

        (updates, removals)
    }
}

impl<S: Storage> Clone for Account<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            index: self.index,
            account: self.account.clone(),
            ops: self.ops.clone(),
        }
    }
}
