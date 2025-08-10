//! Account registry with cached state

use crate::{storage::Commit, Storage};
use anyhow::Result;
use pvm::Gas;
pub use registry::Accounts;
use score::{
    service::{ServiceAccount, ServiceInfo},
    state::account,
    OpaqueHash, TrieKey,
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

    /// The ops of preimages
    preimages: (BTreeSet<OpaqueHash>, BTreeSet<OpaqueHash>),

    /// The ops of preimages
    storage: (BTreeSet<Vec<u8>>, BTreeSet<Vec<u8>>),

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
            preimages: (BTreeSet::new(), BTreeSet::new()),
            storage: (BTreeSet::new(), BTreeSet::new()),
            ops: Commit::default(),
        })
    }

    /// Inherit from another account
    pub fn inherit(storage: Arc<S>, index: u32, account: impl score::Account) -> Self {
        Self {
            state: storage,
            index,
            account: account.account(),
            preimages: (BTreeSet::new(), BTreeSet::new()),
            storage: (BTreeSet::new(), BTreeSet::new()),
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
        self.account.balance
    }

    fn balance_mut(&mut self) -> &mut u64 {
        &mut self.account.balance
    }

    fn blob(&self) -> Option<Vec<u8>> {
        self.account.blob()
    }

    fn code(&self) -> [u8; 32] {
        self.account.code
    }

    fn set_code(&mut self, code: [u8; 32]) {
        self.account.code = code;
    }

    fn accumulate_gas(&self) -> Gas {
        self.account.accumulate_gas
    }

    fn set_accumulate_gas(&mut self, gas: Gas) {
        self.account.accumulate_gas = gas;
    }

    fn transfer_gas(&self) -> Gas {
        self.account.transfer_gas
    }

    fn set_transfer_gas(&mut self, gas: Gas) {
        self.account.transfer_gas = gas;
    }

    fn threshold(&self) -> u64 {
        self.account.threshold()
    }

    fn total(&self) -> u64 {
        self.account.total
    }

    fn set_total(&mut self, total: u64) {
        self.account.set_total(total);
    }

    fn items(&self) -> u32 {
        self.account.items()
    }

    fn creation(&self) -> u32 {
        self.account.creation
    }

    fn set_creation(&mut self, creation: u32) {
        self.account.creation = creation;
    }

    fn update(&self) -> u32 {
        self.account.update
    }

    fn set_update(&mut self, update: u32) {
        self.account.update = update;
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
    }

    fn remove_lookup(&mut self, hash: [u8; 32], len: u32) {
        let key = self.drop_lookup(hash, len);
        self.account.lookup.remove(&(hash, len));
        self.ops.remove(key);
        self.set_total(self.total() - 81 - len as u64);
    }

    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>> {
        self.account.preimage.get(&hash).cloned()
    }

    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        {
            self.preimages.1.remove(&hash);
            let fhash = account::preimage(self.index, hash);
            self.ops.removal.remove(&fhash);
        }

        self.account.preimage.insert(hash, preimage.clone());
        self.preimages.0.insert(hash);
    }

    fn remove_preimage(&mut self, hash: [u8; 32]) {
        self.account.preimage.remove(&hash);
        self.preimages.1.insert(hash);
    }

    fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        let vkey = account::storage(self.index, key).to_vec();
        self.account.storage.get(&vkey)
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        let vkey = account::storage(self.index, key).to_vec();
        {
            if self.storage.1.contains(&vkey) {
                self.storage.1.remove(&vkey);
            }

            let mut fkey = [0; 31];
            fkey.copy_from_slice(&vkey);
            self.ops.removal.remove(&fkey);
        }

        // update total
        if let Some(old) = self.account.storage.get(&vkey).map(|v| v.len() as u64) {
            self.set_total(self.total() + value.len() as u64 - old);
        } else {
            self.set_total(self.total() + 34 + key.len() as u64 + value.len() as u64);
        }

        // update storage
        self.storage.0.insert(vkey.clone());
        self.account.storage.insert(vkey, value);
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let vkey = account::storage(self.index, key).to_vec();
        if !self.storage.1.contains(&vkey) {
            self.storage.1.insert(vkey.clone());
        }

        // update total
        if let Some(old) = self.account.storage.get(&vkey).map(|v| v.len() as u64) {
            self.set_total(self.total() - 34 - key.len() as u64 - old);
        }

        self.account.storage.remove(&vkey)
    }

    fn info(&self) -> ServiceInfo {
        ServiceInfo {
            code: self.account.code,
            balance: self.account.balance,
            transfer: self.account.transfer_gas,
            accumulate: self.account.accumulate_gas,
            total: self.account.total(),
            items: self.account.items(),
            creation: self.account.creation,
            update: self.account.update,
            parent: self.account.parent,
            offset: self.account.offset,
        }
    }

    fn ops(mut self) -> (BTreeMap<TrieKey, Vec<u8>>, BTreeSet<TrieKey>) {
        self.ops.set(
            account::info(self.index),
            codec::encode(&self.account.data()).expect("data is valid"),
        );

        // collect removals
        let mut removals: BTreeSet<TrieKey> = self.ops.iremoval().cloned().collect();
        removals.extend(self.storage.1.iter().map(|k| {
            let mut mkey = [0; 31];
            mkey.copy_from_slice(k);
            mkey
        }));
        removals.extend(
            self.preimages
                .1
                .iter()
                .map(|k| account::preimage(self.index, *k)),
        );

        // collect updates
        let mut updates: BTreeMap<TrieKey, Vec<u8>> =
            self.ops.updates().map(|(k, v)| (k, v.clone())).collect();
        updates.extend(self.storage.0.iter().map(|k| {
            let mut key = [0; 31];
            key.copy_from_slice(k);
            (
                key,
                self.account
                    .storage
                    .get(k)
                    .expect("storage is valid")
                    .clone(),
            )
        }));
        updates.extend(self.preimages.0.iter().map(|k| {
            let mut key = [0; 31];
            key.copy_from_slice(&account::preimage(self.index, *k));
            (
                key,
                self.account
                    .preimage
                    .get(k)
                    .expect("preimage is valid")
                    .clone(),
            )
        }));

        (updates, removals)
    }
}

impl<S: Storage> Clone for Account<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            index: self.index,
            account: self.account.clone(),
            preimages: self.preimages.clone(),
            storage: self.storage.clone(),
            ops: self.ops.clone(),
        }
    }
}
