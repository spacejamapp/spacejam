//! Account registry with cached state

use crate::{storage::Commit, Storage};
use anyhow::Result;
use pvm::score::Gas;
pub use registry::Accounts;
use score::{
    service::{ServiceAccount, ServiceInfo},
    Account as CoreAccount, AccountInnerKey, TrieKey,
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

    /// The info of the account
    info: ServiceInfo,

    /// The account state
    account: ServiceAccount,

    /// The operations of the account
    updates: Commit<AccountInnerKey, Vec<u8>>,
}

impl<S: Storage> Account<S> {
    /// Create a new account
    pub fn new(storage: Arc<S>, index: u32) -> Result<Self> {
        let account = storage.account(index)?;

        Ok(Self {
            state: storage,
            index,
            info: account.info.clone(),
            account,
            updates: Commit::default(),
        })
    }

    /// Read storage via storage key
    pub fn hread(&self, key: &AccountInnerKey) -> Option<Vec<u8>> {
        if self.account.storage.contains_key(key) {
            self.account.storage.get(key).cloned()
        } else {
            self.state.state_get(key.trie()).ok().flatten()
        }
    }

    /// Inherit from another account
    pub fn inherit(storage: Arc<S>, index: u32, account: impl CoreAccount) -> Self {
        let new_info = account.info();
        let new_account = account.account();
        let (update, removal) = account.updates();
        let new_update: BTreeMap<_, _> = update
            .into_iter()
            .map(|(k, v)| (k.inherit(index), v))
            .collect();
        let new_removal: BTreeSet<_> = removal.into_iter().map(|k| k.inherit(index)).collect();
        Self {
            state: storage,
            index,
            info: new_info,
            account: new_account,
            updates: (new_update, new_removal).into(),
        }
    }
}

impl<S: Storage> CoreAccount for Account<S> {
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
        let ikey = AccountInnerKey::Lookup(self.index, hash, len);
        if let Some(lookup) = self.account.lookup.get(&ikey) {
            return Some(lookup.clone());
        }

        // Check if this key is marked for removal in the current transaction
        if self.updates.removal.contains(&ikey) {
            return None;
        }

        if let Some(lookup) = self.state.state_get(ikey.trie()).ok().flatten() {
            let lookup: Vec<u32> = codec::decode(&lookup).ok()?;
            self.account.lookup.insert(ikey, lookup.clone());
            Some(lookup)
        } else {
            None
        }
    }

    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, lookup: Vec<u32>) {
        let ikey = AccountInnerKey::Lookup(self.index, hash, len);
        let exists = self.state.state_get(ikey.trie()).ok().flatten().is_some();
        self.account.lookup.insert(ikey.clone(), lookup.clone());

        self.updates.removal.remove(&ikey);
        let encoded = codec::encode(&lookup).expect("lookup is valid");
        self.updates.set(ikey, encoded);

        // Only update footprint if this is a new lookup entry:
        //
        //  a_i = 2 * |a_l| + |a_s| (items)
        //  a_o includes Σ(81 + z) for each lookup (total octets)
        if !exists {
            self.set_total(self.total() + 81 + len as u64);
            self.set_items(self.items() + 2);
        }
    }

    fn remove_lookup(&mut self, hash: [u8; 32], len: u32) {
        let ikey = AccountInnerKey::Lookup(self.index, hash, len);
        self.account.lookup.remove(&ikey);
        if self.state.state_get(ikey.trie()).ok().flatten().is_some() {
            self.updates.remove(ikey);
            self.set_total(self.total() - 81 - len as u64);
            self.set_items(self.items() - 2);
        }
    }

    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>> {
        let ikey = AccountInnerKey::Preimage(self.index, hash);
        self.account
            .preimage
            .get(&ikey)
            .cloned()
            .or_else(|| self.state.state_get(ikey.trie()).ok().flatten())
    }

    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        let ikey = AccountInnerKey::Preimage(self.index, hash);
        self.account.preimage.insert(ikey.clone(), preimage.clone());
        self.updates.set(ikey, preimage);
    }

    fn remove_preimage(&mut self, hash: [u8; 32]) {
        let ikey = AccountInnerKey::Preimage(self.index, hash);
        self.account.preimage.remove(&ikey);
        self.updates.update.remove(&ikey);
        self.updates.remove(ikey);
    }

    fn read(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let ikey = AccountInnerKey::Storage(self.index, key.to_vec());
        if self.updates.removal.contains(&ikey) {
            return None;
        }

        self.hread(&ikey)
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        let ikey = AccountInnerKey::Storage(self.index, key.to_vec());
        let mut previous = None;
        if let Some(old) = self.hread(&ikey) {
            if self.updates.removal.contains(&ikey) {
                self.set_items(self.items() + 1);
                self.set_total(self.total() + 34 + key.len() as u64 + value.len() as u64);
            } else {
                self.set_total(self.total() + value.len() as u64 - old.len() as u64);
            }

            previous = Some(old);
        } else {
            self.set_items(self.items() + 1);
            self.set_total(self.total() + 34 + key.len() as u64 + value.len() as u64);
        }

        // update storage
        self.updates.removal.remove(&ikey);
        self.updates.set(ikey.clone(), value.clone());
        self.account
            .storage
            .insert(ikey.clone(), value)
            .or(previous);
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let ikey = AccountInnerKey::Storage(self.index, key.to_vec());
        if self.updates.removal.contains(&ikey) {
            return None;
        }

        // update total
        let mut removed = None;
        if let Some(old) = self.hread(&ikey) {
            self.set_total(self.total() - 34 - key.len() as u64 - old.len() as u64);
            self.set_items(self.items() - 1);
            self.updates.remove(ikey.clone());
            removed = Some(old);
        }

        self.account.storage.remove(&ikey);
        removed
    }

    fn info(&self) -> ServiceInfo {
        self.account.info.clone()
    }

    fn updated(&self) -> bool {
        !self.updates.is_empty() || self.info != self.account.info
    }

    fn updates(
        mut self,
    ) -> (
        BTreeMap<AccountInnerKey, Vec<u8>>,
        BTreeSet<AccountInnerKey>,
    ) {
        if self.info != self.account.info {
            self.updates.set(
                AccountInnerKey::Info(self.index),
                codec::encode(&self.account.state()).expect("data is valid"),
            );
        }
        let Commit { update, removal } = self.updates;

        (update, removal)
    }

    fn ops(mut self) -> (BTreeMap<TrieKey, Vec<u8>>, BTreeSet<TrieKey>) {
        if self.info != self.account.info {
            self.updates.set(
                AccountInnerKey::Info(self.index),
                codec::encode(&self.account.state()).expect("data is valid"),
            );
        }

        let Commit { update, removal } = self.updates;
        let removals: BTreeSet<TrieKey> = removal.into_iter().map(|k| k.trie()).collect();
        let updates: BTreeMap<TrieKey, Vec<u8>> =
            update.into_iter().map(|(k, v)| (k.trie(), v)).collect();

        (updates, removals)
    }
}

impl<S: Storage> Clone for Account<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            index: self.index,
            info: self.info.clone(),
            account: self.account.clone(),
            updates: self.updates.clone(),
        }
    }
}
