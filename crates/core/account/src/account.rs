//! Account abstraction

use score::{
    Gas, OpaqueHash, TrieKey,
    service::{ServiceAccount, ServiceInfo},
};
use std::collections::{BTreeMap, BTreeSet};

/// JAM account abstraction
pub trait Account: Clone {
    /// Get the index of the account
    fn index(&self) -> u32;

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

    /// Set the total of the account
    fn set_total(&mut self, total: u64);

    /// Get the items of the account
    fn items(&self) -> u32;

    /// Set the items of the account
    fn set_items(&mut self, items: u32);

    /// Get the code of the account
    fn code(&self) -> OpaqueHash;

    /// Set the code of the account
    fn set_code(&mut self, code: OpaqueHash);

    /// Get the accumulate gas of the account
    fn accumulate_gas(&self) -> Gas;

    /// Set the accumulate gas of the account
    fn set_accumulate_gas(&mut self, gas: Gas);

    /// Get the transfer gas of the account
    fn transfer_gas(&self) -> Gas;

    /// Set the transfer gas of the account
    fn set_transfer_gas(&mut self, gas: Gas);

    /// Get the creation time of the account
    fn creation(&self) -> u32;

    /// Set the creation time of the account
    fn set_creation(&mut self, creation: u32);

    /// Get the last update time of the account
    fn update(&self) -> u32;

    /// Set the last update time of the account
    fn set_update(&mut self, update: u32);

    /// Get a lookup from the account
    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Option<Vec<u32>>>;

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
    fn read(&mut self, key: &[u8]) -> Option<Vec<u8>>;

    /// Remove a storage from the account
    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>>;

    /// Write a storage to the account
    fn write(&mut self, key: &[u8], value: Vec<u8>);

    /// Get the account info
    fn info(&self) -> ServiceInfo;

    /// Check if the account has been updated
    fn updated(&self) -> bool {
        false
    }

    /// Get the operations of the account
    fn ops(self) -> (BTreeMap<TrieKey, Vec<u8>>, BTreeSet<TrieKey>);

    /// Get the threshold with new lookup
    fn lookup_threshold(&self, len: u64) -> Option<u64> {
        self.threshold()
            .checked_add(
                81u64
                    .checked_add(len)?
                    .checked_mul(score::BALANCE_PER_OCTET)?,
            )?
            .checked_add(2u64.checked_mul(score::BALANCE_PER_ITEM)?)
    }

    /// Write the threshold of the account
    fn write_threshold(&self, key: &[u8], value: &[u8]) -> Option<u64> {
        let klen = key.len() as u64;
        let vlen = value.len() as u64;
        self.threshold()
            .checked_add(score::BALANCE_PER_ITEM + (34 + klen + vlen) * score::BALANCE_PER_OCTET)
    }

    /// (Λ) lookup preimage in the recent histories
    fn historical_lookup(&mut self, timeslot: u32, hash: [u8; 32]) -> Option<Vec<u8>> {
        let preimage = self.preimage(hash)?;
        let lookup = self.lookup(hash, preimage.len() as u32).flatten()?;
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

    /// Add a preimage to the account
    #[cfg(feature = "blake2")]
    fn add_preimage(&mut self, preimage: Vec<u8>, timeslot: u32) -> score::OpaqueHash {
        let hash = crypto::blake2b(&preimage);
        self.insert_lookup(hash, preimage.len() as u32, vec![timeslot]);
        self.insert_preimage(hash, preimage);
        hash
    }
}

impl Account for ServiceAccount {
    fn index(&self) -> u32 {
        self.index
    }

    fn account(&self) -> ServiceAccount {
        self.clone()
    }

    fn balance(&self) -> u64 {
        self.info.balance
    }

    fn balance_mut(&mut self) -> &mut u64 {
        &mut self.info.balance
    }

    fn blob(&self) -> Option<Vec<u8>> {
        self.preimage.get(&self.info.code).cloned()
    }

    fn threshold(&self) -> u64 {
        self.info.threshold()
    }

    fn total(&self) -> u64 {
        self.info.total
    }

    fn set_total(&mut self, total: u64) {
        self.info.total = total;
    }

    fn items(&self) -> u32 {
        self.info.items
    }

    fn set_items(&mut self, items: u32) {
        self.info.items = items;
    }

    fn code(&self) -> OpaqueHash {
        self.info.code
    }

    fn set_code(&mut self, code: OpaqueHash) {
        self.info.code = code;
    }

    fn accumulate_gas(&self) -> Gas {
        self.info.accumulate
    }

    fn set_accumulate_gas(&mut self, gas: Gas) {
        self.info.accumulate = gas;
    }

    fn transfer_gas(&self) -> Gas {
        self.info.transfer
    }

    fn set_transfer_gas(&mut self, gas: Gas) {
        self.info.transfer = gas;
    }

    fn creation(&self) -> u32 {
        self.info.creation
    }

    fn set_creation(&mut self, creation: u32) {
        self.info.creation = creation;
    }

    fn update(&self) -> u32 {
        self.info.update
    }

    fn set_update(&mut self, update: u32) {
        self.info.update = update;
    }

    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Option<Vec<u32>>> {
        self.lookup.get(&(hash, len)).cloned().map(Some)
    }

    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, slots: Vec<u32>) {
        self.set_total(self.total() + 81 + len as u64);
        self.set_items(self.items() + 2);
        self.lookup.insert((hash, len), slots);
    }

    fn remove_lookup(&mut self, hash: [u8; 32], len: u32) {
        self.set_total(self.total() - 81 - len as u64);
        self.set_items(self.items() - 2);
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

    fn read(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let skey = score::state::account::storage(self.index(), key);
        self.storage.get(skey.as_slice()).cloned()
    }

    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let skey = score::state::account::storage(self.index(), key);
        let value = self.storage.remove(skey.as_slice())?;
        self.set_total(self.total() - 34 - key.len() as u64 - value.len() as u64);
        self.set_items(self.items() - 1);
        Some(value)
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        let skey = score::state::account::storage(self.index(), key);
        if let Some(old) = self.storage.get(skey.as_slice()).map(|v| v.len() as u64) {
            self.set_total(self.total() + value.len() as u64 - old);
        } else {
            self.set_total(self.total() + 34 + key.len() as u64 + value.len() as u64);
            self.set_items(self.items() + 1);
        }

        self.storage.insert(skey.to_vec(), value);
    }

    fn info(&self) -> ServiceInfo {
        self.state()
    }

    fn ops(self) -> (BTreeMap<TrieKey, Vec<u8>>, BTreeSet<TrieKey>) {
        let mut updates = BTreeMap::new();
        let removals = BTreeSet::new();

        // Ensure the account info is written to storage
        let info_key = score::state::account::info(self.index);
        let encoded_info = codec::encode(&self.info).expect("service info is valid");
        updates.insert(info_key, encoded_info);

        // Ensure the lookup is written to storage
        for ((hash, len), slots) in self.lookup.iter() {
            let lookup_key = score::state::account::lookup(self.index, *len, *hash);
            let encoded_lookup = codec::encode(slots).expect("lookup is valid");
            updates.insert(lookup_key, encoded_lookup);
        }

        (updates, removals)
    }
}
