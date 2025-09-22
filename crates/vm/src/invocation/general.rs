//! General host functions

use crate::Argument;
use score::{vm::Operand, Entropy, ServiceId};
use scorext::{Account, Accounts};

/// Input data of general host functions
#[derive(Debug, Clone)]
pub struct General<R: Accounts> {
    /// (s) Service index
    pub index: ServiceId,

    /// (d) Account dictionary
    pub accounts: R,

    /// (o) The operands
    pub operands: Vec<Operand>,

    /// (η) The entropy
    pub entropy: Entropy,

    // if the account got updated.
    pub updated: bool,
}

impl<R: Accounts> General<R> {
    /// Create a new general host
    pub fn new(index: ServiceId, accounts: R, operands: Vec<Operand>, entropy: Entropy) -> Self {
        Self {
            index,
            accounts,
            operands,
            entropy,
            updated: false,
        }
    }

    /// Get service account
    pub fn get(&mut self, r7: u64) -> Option<impl Account + '_> {
        let service = self.index as u64;
        let mut index = r7 as ServiceId;
        if r7 == u64::MAX || r7 == service {
            index = self.index;
        }

        self.accounts.get(index).cloned()
    }

    /// Get the account
    pub fn account(&mut self) -> Option<&mut (impl Account + '_)> {
        self.accounts.get(self.index)
    }
}

impl<R: Accounts> Argument for General<R> {
    const SUPPORTED_CALLS: &[u32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

    const INITIAL_PC: u64 = 0;

    fn account(&mut self, id: u64) -> anyhow::Result<&mut impl Account> {
        self.accounts
            .get(id as u32)
            .ok_or(anyhow::anyhow!("Could not find account {id}"))
    }

    fn this(&mut self) -> anyhow::Result<&mut impl Account> {
        self.accounts
            .get(self.index)
            .ok_or(anyhow::anyhow!("Could not find account {}", self.index))
    }
}
