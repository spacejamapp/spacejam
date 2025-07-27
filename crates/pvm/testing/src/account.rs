//! Service account builder

use crate::Jam;
use anyhow::Result;
use score::{service::ServiceAccount, Account, OpaqueHash, ServiceId};

impl Jam {
    /// Add a service account
    pub fn add_account(&mut self, service: ServiceId, account: ServiceAccount) {
        self.chain.accounts.insert(service, account);
    }

    /// Add a preimage to the service account
    pub fn add_preimage(&mut self, service: ServiceId, preimage: Vec<u8>) -> Result<OpaqueHash> {
        let account = self.chain.accounts.entry(service).or_default();
        let hash = account.add_preimage(preimage, self.chain.finalized.slot);
        Ok(hash)
    }

    /// Mint balance to a service account
    pub fn mint(&mut self, service: ServiceId, amount: u64) {
        let account = self.chain.accounts.entry(service).or_default();
        account.balance += amount;
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
