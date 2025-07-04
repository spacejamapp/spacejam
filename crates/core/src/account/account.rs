//! Account abstraction

use crate::{
    service::{GasLimit, ServiceAccount, ServiceInfo},
    OpaqueHash, TrieKey,
};
use std::collections::{BTreeMap, BTreeSet};

/// JAM account abstraction
pub trait Account: Clone {
    /// Get the account
    fn account(&self) -> ServiceAccount;

    /// Get the balance of the account
    fn balance(&self) -> u64;

    /// Get the balance of the account
    fn balance_mut(&mut self) -> &mut u64;

    /// Get the blob of the account
    fn blob(&self) -> Option<Vec<u8>>;

    /// Get the threshold of the account
    fn threshold(&self) -> u64;

    /// Get the total of the account
    fn total(&self) -> u64;

    /// Get the items of the account
    fn items(&self) -> u32;

    /// Get the code of the account
    fn code(&self) -> OpaqueHash;

    /// Set the code of the account
    fn set_code(&mut self, code: OpaqueHash);

    /// Get the gas of the account
    fn gas(&self) -> GasLimit;

    /// Set the gas of the account
    fn set_gas(&mut self, gas: GasLimit);

    /// Get a lookup from the account
    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Vec<u32>>;

    /// (Λ) lookup preimage in the recent histories
    fn historical_lookup(&mut self, timeslot: u32, hash: [u8; 32]) -> Option<Vec<u8>> {
        let preimage = self.preimage(hash)?;
        let lookup = self.lookup(hash, preimage.len() as u32)?;
        if (lookup.len() == 1 && timeslot >= lookup[0])
            || (lookup.len() == 2 && timeslot >= lookup[0] && timeslot <= lookup[1])
            || (lookup.len() == 3
                && ((timeslot >= lookup[0] && timeslot < lookup[1]) || timeslot >= lookup[2]))
        {
            Some(preimage)
        } else {
            None
        }
    }

    /// Insert a lookup to the account
    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, slots: Vec<u32>);

    /// Remove a lookup from the account
    fn remove_lookup(&mut self, hash: [u8; 32], len: u32);

    /// Get a preimage from the account
    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>>;

    /// Insert a preimage to the account
    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>);

    /// Remove a preimage from the account
    fn remove_preimage(&mut self, hash: [u8; 32]);

    /// Get a storage from the account
    fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>>;

    /// Remove a storage from the account
    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>>;

    /// Write a storage to the account
    fn write(&mut self, key: &[u8], value: Vec<u8>);

    /// Get the account info
    fn info(&self) -> ServiceInfo;

    /// Get the operations of the account
    fn ops(self) -> (BTreeMap<TrieKey, Vec<u8>>, BTreeSet<TrieKey>);
}

impl Account for ServiceAccount {
    fn account(&self) -> ServiceAccount {
        self.clone()
    }

    fn balance(&self) -> u64 {
        self.balance
    }

    fn balance_mut(&mut self) -> &mut u64 {
        &mut self.balance
    }

    fn blob(&self) -> Option<Vec<u8>> {
        self.preimage.get(&self.code).cloned()
    }

    fn threshold(&self) -> u64 {
        self.threshold()
    }

    fn total(&self) -> u64 {
        self.total()
    }

    fn items(&self) -> u32 {
        self.items()
    }

    fn code(&self) -> OpaqueHash {
        self.code
    }

    fn set_code(&mut self, code: OpaqueHash) {
        self.code = code;
    }

    fn gas(&self) -> GasLimit {
        self.gas.clone()
    }

    fn set_gas(&mut self, gas: GasLimit) {
        self.gas = gas;
    }

    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Vec<u32>> {
        self.lookup.get(&(hash, len)).cloned()
    }

    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, slots: Vec<u32>) {
        self.lookup.insert((hash, len), slots);
    }

    fn remove_lookup(&mut self, hash: [u8; 32], len: u32) {
        self.lookup.remove(&(hash, len));
    }

    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>> {
        self.preimage.get(&hash).cloned()
    }

    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        self.preimage.insert(hash, preimage);
    }

    fn remove_preimage(&mut self, hash: [u8; 32]) {
        self.preimage.remove(&hash);
    }

    fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.remove(key)
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        self.storage.insert(key.to_vec(), value);
    }

    fn info(&self) -> ServiceInfo {
        self.state()
    }

    fn ops(self) -> (BTreeMap<TrieKey, Vec<u8>>, BTreeSet<TrieKey>) {
        (BTreeMap::new(), BTreeSet::new())
    }
}
