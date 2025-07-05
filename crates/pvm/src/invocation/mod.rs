//! PVM invocation interface

use score::{vm::Operand, Account, Accounts, ServiceId};
pub use {
    accumulate::Accumulate,
    api::Invocation,
    refine::Refine,
    state::{Executed, Received, State, Stepped},
};

pub mod accumulate;
mod api;
pub mod refine;
mod state;
pub mod transfer;

/// Dynamic arguments for host calls
pub trait Argument<R: Accounts> {
    /// returns some if the input data is general
    fn as_general(&self) -> crate::Result<General<R>> {
        crate::bail!("not a general")
    }

    /// update the general argument
    fn update_general(&mut self, _general: General<R>) -> crate::Result<()> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is accumulate
    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate<R>> {
        crate::bail!("not an accumulate")
    }

    /// returns some if the input data is refine
    fn as_refine_mut(&mut self) -> crate::Result<&mut Refine<R>> {
        crate::bail!("not a refine")
    }

    /// returns the arguments of the invocation
    fn args(&self) -> &[u8] {
        &[]
    }
}

/// Input data of general host functions
#[derive(Debug, Clone)]
pub struct General<R: Accounts> {
    /// (s) Service index
    pub index: ServiceId,

    /// (d) Account dictionary
    pub accounts: R,

    // if the account got updated.
    pub updated: bool,

    /// (o) The operands
    pub operands: Vec<Operand>,
}

impl<R: Accounts> General<R> {
    /// Create a new general host
    pub fn new(index: ServiceId, accounts: R, operands: Vec<Operand>) -> Self {
        Self {
            index,
            accounts,
            updated: false,
            operands,
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

impl<R: Accounts> Argument<R> for () {}
