//! Service account builder

use crate::Jam;
use score::{Account, ServiceId};

impl Jam {
    /// Mint balance to a service account
    pub fn mint(&mut self, service: ServiceId, amount: u64) {
        let account = self.chain.accounts.entry(service).or_default();
        account.balance += amount;
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
