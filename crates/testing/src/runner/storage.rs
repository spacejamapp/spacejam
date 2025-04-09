//! Account storage for testing
#![allow(dead_code)]

use anyhow::Result;
use score::service::{ServiceAccountData, ServiceItem};
use spacejam::Storage;

/// Extension trait for the `Storage` trait
pub trait StorageExt: Storage {
    /// Add an account to the storage
    fn add_account(&self, id: u32, account: &ServiceAccountData) -> Result<()> {
        self.set_info(id, &account.service)?;

        for preimage in &account.preimages {
            self.set_preimage(id, preimage.hash, preimage.blob.clone())?;
        }
        Ok(())
    }

    fn add_accounts(&self, accounts: Vec<ServiceItem>) -> Result<()> {
        for account in accounts {
            self.add_account(account.id, &account.data)?;
        }
        Ok(())
    }
}

impl<S: Storage> StorageExt for S {}
