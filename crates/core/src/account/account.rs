use crate::service::{ServiceAccount, ServiceAccountState};

/// JAM account abstraction
pub trait Account {
    /// Get a lookup from the account
    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Vec<u32>>;

    /// Insert a lookup to the account
    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, slots: Vec<u32>);

    /// Get a preimage from the account
    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>>;

    /// Insert a preimage to the account
    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>);

    /// Get a storage from the account
    fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>>;

    /// Remove a storage from the account
    fn remove(&mut self, key: &[u8]);

    /// Write a storage to the account
    fn write(&mut self, key: &[u8], value: Vec<u8>);

    /// Get the account info
    fn info(&self) -> ServiceAccountState;
}

impl Account for ServiceAccount {
    fn lookup(&mut self, hash: [u8; 32], len: u32) -> Option<Vec<u32>> {
        self.lookup.get(&(hash, len)).cloned()
    }

    fn insert_lookup(&mut self, hash: [u8; 32], len: u32, slots: Vec<u32>) {
        self.lookup.insert((hash, len), slots);
    }

    fn preimage(&mut self, hash: [u8; 32]) -> Option<Vec<u8>> {
        self.preimage.get(&hash).cloned()
    }

    fn insert_preimage(&mut self, hash: [u8; 32], preimage: Vec<u8>) {
        self.preimage.insert(hash, preimage);
    }

    fn read(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }

    fn remove(&mut self, key: &[u8]) {
        self.storage.remove(key);
    }

    fn write(&mut self, key: &[u8], value: Vec<u8>) {
        self.storage.insert(key.to_vec(), value);
    }

    fn info(&self) -> ServiceAccountState {
        self.state()
    }
}
