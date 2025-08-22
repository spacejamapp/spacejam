//! Argument for host calls

use anyhow::Result;
use score::{
    safrole::ValidatorData,
    service::Privileges,
    vm::{DeferredTransfer, Operand},
    Account, OpaqueHash, ServiceId, TimeSlot,
};

/// Dynamic arguments for host calls
pub trait Argument {
    const SUPPORTED_CALLS: &[u32];

    /// Get an account by account id
    fn account(&mut self, id: u64) -> Result<&mut impl Account>;

    /// Get the check index
    fn check(&mut self, index: ServiceId) -> ServiceId {
        index
    }

    /// Make a checkpoint
    fn checkpoint(&mut self) {}

    /// Get the entropy (η'0)
    fn entropy(&self) -> OpaqueHash {
        OpaqueHash::default()
    }

    /// Get the free index
    fn index(&self) -> ServiceId {
        0
    }

    /// Get the operands
    fn operands(&self) -> &[Operand] {
        &[]
    }

    /// Get the account or this
    fn or_this(&mut self, account: u64) -> Result<&mut impl Account> {
        let service = self.service() as u64;
        let mut index = account;
        if account == u64::MAX || account == service {
            index = service;
        }

        self.account(index)
    }

    /// Set the output hash
    fn output(&mut self, hash: OpaqueHash) {
        let _ = hash;
    }

    /// Get the privileges
    fn privileges(&self) -> Privileges {
        Privileges::default()
    }

    /// Remove an account
    fn remove(&mut self, service: ServiceId) {
        let _ = service;
    }

    /// Get the service index
    fn service(&self) -> ServiceId {
        0
    }

    /// Set the service index
    fn set_index(&mut self, index: ServiceId) {
        let _ = index;
    }

    /// Set the authorization queue
    fn set_authorization(&mut self, core: u16, queue: Vec<[u8; 32]>) {
        let _ = (core, queue);
    }

    /// Set the assign queue
    fn set_assign(&mut self, core: u16, assign: ServiceId) {
        let _ = (core, assign);
    }

    /// Set the privileges
    fn set_privileges(&mut self, privileges: Privileges) {
        let _ = privileges;
    }

    /// Set the validators
    fn set_validators(&mut self, validators: [ValidatorData; score::VALIDATORS_COUNT as usize]) {
        let _ = validators;
    }

    /// Get the service account
    fn this(&mut self) -> Result<&mut impl Account>;

    /// Get the timeslot
    fn timeslot(&self) -> TimeSlot {
        0
    }

    /// Transfer a deferred transfer
    fn transfer(&mut self, transfer: DeferredTransfer) {
        let _ = transfer;
    }

    /// Update the account
    fn update(&mut self, account: ServiceId) {
        let _ = account;
    }

    /// Upsert an account
    fn upsert(&mut self, id: ServiceId, account: impl Account) {
        let _ = (id, account);
    }
}
