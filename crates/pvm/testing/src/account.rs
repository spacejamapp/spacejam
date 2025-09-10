//! Service account builder

use crate::Jam;
use score::{service::ServiceAccount, Account, AccountInnerKey, OpaqueHash, ServiceId};

impl Jam {
    /// Add a service account
    pub fn add_account(&mut self, service: ServiceId, account: ServiceAccount) {
        self.chain.accounts.insert(service, account);
    }

    /// Add a service account
    pub fn add_service(&mut self, service: ServiceId, code: Vec<u8>) {
        let hash = self.add_preimage(service, code);
        self.set_code(service, hash);
        self.mint(service, 1_000_000_000);
    }

    /// Add a preimage to the service account
    pub fn add_preimage(&mut self, service: ServiceId, preimage: Vec<u8>) -> OpaqueHash {
        let account = self.chain.accounts.entry(service).or_default();
        account.add_preimage(preimage, self.chain.finalized.slot)
    }

    /// Get a storage of an account
    pub fn get_storage<V: podec::Decode>(&self, service: ServiceId, key: &[u8]) -> Option<V> {
        let account = self.chain.accounts.get(&service)?;
        let encoded = account
            .storage
            .get(&AccountInnerKey::Storage(service, key.to_vec()))?;
        V::decode(&mut &encoded[..]).ok()
    }

    /// Set the code of the service account
    pub fn set_code(&mut self, service: ServiceId, code: OpaqueHash) {
        let account = self.chain.accounts.entry(service).or_default();
        account.set_code(code);
    }

    /// Mint balance to a service account
    pub fn mint(&mut self, service: ServiceId, amount: u64) {
        let account = self.chain.accounts.entry(service).or_default();
        account.info.balance += amount;
    }

    /// Add a service account
    pub fn with_account(mut self, service: ServiceId, account: ServiceAccount) -> Self {
        self.chain.accounts.insert(service, account);
        self
    }
}

/// Service account builder
pub trait AccountBuilder: Account {
    /// Set the balance of the account
    fn with_balance(mut self, balance: u64) -> Self {
        *self.balance_mut() = balance;
        self
    }

    /// Set the code of the account
    fn with_preimage(mut self, preimage: Vec<u8>, timeslot: u32) -> Self {
        self.add_preimage(preimage, timeslot);
        self
    }
}

impl<T> AccountBuilder for T where T: Account {}
