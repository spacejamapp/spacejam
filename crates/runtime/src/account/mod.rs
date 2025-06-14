//! Account registry with cached state

use crate::{Storage, storage::Commit};
use anyhow::Result;
pub use registry::Accounts;
use score::{
    OpaqueHash, StorageKey,
    service::{GasLimit, ServiceAccount, ServiceAccountState},
    state::account,
};
use std::{collections::BTreeSet, sync::Arc};

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
    ops: Commit<StorageKey, Vec<u8>>,
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
    pub fn inherit(storage: Arc<S>, index: u32, account: impl score::account::Account) -> Self {
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
    pub fn drop_lookup(&mut self, hash: [u8; 32], len: u32) -> StorageKey {
        let key = account::lookup(self.index, len, hash);
        let mut mhash = [0; 32];
        mhash[..31].copy_from_slice(&key);
        self.account.lookup.remove(&(mhash, len));
        key
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
        self.account.blob()
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
        self.ops
            .set(key, codec::encode(&lookup).expect("lookup is valid"));
    }

    fn remove_lookup(&mut self, hash: [u8; 32], len: u32) {
        let key = self.drop_lookup(hash, len);
        self.account.lookup.remove(&(hash, len));
        self.ops.remove(key)
    }

    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>> {
        self.account.preimage.get(&hash).cloned()
    }

    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        self.account.preimage.insert(hash, preimage.clone());
        self.preimages.0.insert(hash);
    }

    fn remove_preimage(&mut self, hash: [u8; 32]) {
        self.account.preimage.remove(&hash);
        self.preimages.1.insert(hash);
    }

    fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        self.account.storage.get(key)
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        self.storage.0.insert(key.to_vec());
        self.account.storage.insert(key.to_vec(), value);
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.1.insert(key.to_vec());
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

    fn ops(self) -> (BTreeSet<(StorageKey, Vec<u8>)>, BTreeSet<StorageKey>) {
        let mut removals: BTreeSet<StorageKey> = self.ops.iremoval().cloned().collect();
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

        // updates
        let mut updates: BTreeSet<(StorageKey, Vec<u8>)> =
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

        // embed the data, we update it always
        updates.insert((
            account::info(self.index),
            codec::encode(&self.account.data()).expect("data is valid"),
        ));

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
