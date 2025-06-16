//! General host call arguments

use crate::invocation::Argument;
use score::{Account, Accounts, ServiceId};

/// Input data of general host functions
#[derive(Debug, Clone)]
pub struct General<R: Accounts> {
    /// (s) Service index
    pub index: ServiceId,

    /// (d) Account dictionary
    pub accounts: R,

    // if the account got updated.
    pub updated: bool,
}

impl<R: Accounts> General<R> {
    /// Create a new general host
    pub fn new(index: ServiceId, accounts: R) -> Self {
        Self {
            index,
            accounts,
            updated: false,
        }
    }

    /// Get service account
    pub fn get(&mut self, r7: u64) -> Option<(ServiceId, impl Account + '_)> {
        let service = self.index as u64;
        let mut index = r7 as ServiceId;
        if r7 == u64::MAX || r7 == service {
            index = self.index;
        }

        self.accounts
            .get(index)
            .map(|account| (index, account.clone()))
    }

    /// Get the account
    pub fn account(&mut self) -> Option<&mut (impl Account + '_)> {
        self.accounts.get(self.index)
    }
}

impl<R: Accounts> Argument<R> for General<R> {
    fn as_general(&self) -> crate::Result<General<R>> {
        Ok(self.clone())
    }

    fn update_general(&mut self, general: General<R>) -> crate::Result<()> {
        *self = general;
        Ok(())
    }
}
