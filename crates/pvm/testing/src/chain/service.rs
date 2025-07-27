//! Service API

use crate::Jam;
use anyhow::Result;
use pvm::Account;
use score::{service::ServiceAccount, OpaqueHash, ServiceId};

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

    /// Add a service account
    pub fn with_account(mut self, service: ServiceId, account: ServiceAccount) -> Self {
        self.chain.accounts.insert(service, account);
        self
    }
}
