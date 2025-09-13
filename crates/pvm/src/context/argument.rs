//! Argument for host calls

use anyhow::Result;
use score::{
    safrole::ValidatorData,
    service::{Privileges, ServiceAccount},
    vm::{DeferredTransfer, Operand},
    Account, Gas, OpaqueHash, ServiceId, TimeSlot,
};

/// Dynamic arguments for host calls
pub trait Argument: Send + Sync {
    /// Supported host calls
    const SUPPORTED_CALLS: &[u32];

    /// The initial program counter for execution
    const INITIAL_PC: u64;

    /// Get an account by account id
    fn account(&mut self, id: u64) -> Result<&mut impl Account>;

    /// Get the output of the accumulation
    fn acc_output(&mut self) -> Vec<u8> {
        let ptr = self.rget(7) as u32;
        let len = self.rget(8) as u32;
        self.read(ptr, len).unwrap_or_default()
    }

    /// Burn the input gas
    fn burn(&mut self, gas: Gas) {
        unimplemented!("make sure your are invoking the accumulation interface: gas={gas}")
    }

    /// Get the check index
    fn check(&mut self, index: ServiceId) -> ServiceId {
        unimplemented!("make sure your are invoking the accumulation interface: index={index}")
    }

    /// Make a checkpoint
    fn checkpoint(&mut self) {
        unimplemented!("make sure your are invoking the accumulation interface")
    }

    /// Get the entropy (η'0)
    fn entropy(&self) -> OpaqueHash {
        unimplemented!("make sure your are invoking the accumulation interface")
    }

    /// Get the gas
    fn gas(&self) -> Gas {
        unimplemented!("make sure your are invoking the accumulation interface")
    }

    /// Get the free index
    fn index(&self) -> ServiceId {
        unimplemented!("make sure your are invoking the accumulation interface")
    }

    /// Get the operands
    fn operands(&self) -> &[Operand] {
        unimplemented!("make sure your are invoking the accumulation interface")
    }

    /// Get the account or this
    fn or_this(&mut self, account: u64) -> Result<&mut impl Account> {
        let service = self.service() as u64;
        let mut index = account;
        if account == u64::MAX {
            index = service;
        }

        self.account(index)
    }

    /// Set the output hash
    fn output(&mut self, hash: OpaqueHash) {
        unimplemented!("make sure you are invoking the accumulation interface: hash={hash:?}")
    }

    /// Get the privileges
    fn privileges(&self) -> Privileges {
        Privileges::default()
    }

    /// Get the register value
    fn rget(&self, reg: u8) -> u64 {
        unimplemented!("make sure you are invoking the accumulation interface: reg={reg}")
    }

    /// Set the register value
    fn rset(&mut self, reg: u8, value: u64) {
        unimplemented!(
            "make sure you are invoking the accumulation interface: reg={reg} value={value}"
        )
    }

    /// Remove an account
    fn remove(&mut self, service: ServiceId) {
        unimplemented!("make sure you are invoking the accumulation interface: service={service}")
    }

    /// The sbrk instruction
    fn sbrk(&mut self, target: u8, increment: u8) {
        let increment = self.rget(increment);
        let heap_ptr = self.heap_ptr();
        self.rset(target, heap_ptr as u64);
        if increment == 0u64 {
            return;
        }

        let funp = |x: u64| x.div_ceil(crate::PAGE_SIZE) * crate::PAGE_SIZE;
        let boundary = funp(self.heap_ptr() as u64);
        let nptr = self.heap_ptr() as u64 + increment;
        if nptr > boundary {
            let start = boundary / crate::PAGE_SIZE;
            let count = funp(nptr) / crate::PAGE_SIZE - start;
            let _ = self.allocate(start as u32, count as u32);
        }

        self.set_heap_ptr(heap_ptr + increment as u32);
    }

    /// Get the service index
    fn service(&self) -> ServiceId {
        unimplemented!("make sure you are invoking the accumulation interface")
    }

    /// Set the service index
    fn set_index(&mut self, index: ServiceId) {
        unimplemented!("make sure you are invoking the accumulation interface: index={index}")
    }

    /// Set the authorization queue
    fn set_authorization(&mut self, core: u16, queue: Vec<[u8; 32]>) {
        unimplemented!(
            "make sure you are invoking the accumulation interface core={core} queue={queue:?}"
        );
    }

    /// Set the assign queue
    fn set_assign(&mut self, core: u16, assign: ServiceId) {
        unimplemented!(
            "make sure you are invoking the accumulation interface core={core} assign={assign}",
        );
    }

    /// Set the heap pointer
    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        unimplemented!("make sure you are invoking the accumulation interface: heap_ptr={heap_ptr}")
    }

    /// Set the privileges
    fn set_privileges(&mut self, privileges: Privileges) {
        unimplemented!(
            "make sure you are invoking the accumulation interface {:?}",
            privileges
        );
    }

    /// Set the validators
    fn set_validators(&mut self, validators: [ValidatorData; score::VALIDATORS_COUNT as usize]) {
        unimplemented!(
            "make sure you are invoking the accumulation interface {:?}",
            validators.len()
        );
    }

    /// Get the service account
    fn this(&mut self) -> Result<&mut impl Account>;

    /// Get the timeslot
    fn timeslot(&self) -> TimeSlot {
        unimplemented!("make sure you are invoking the accumulation interface")
    }

    /// Transfer a deferred transfer
    fn transfer(&mut self, transfer: DeferredTransfer) {
        unimplemented!(
            "make sure you are invoking the accumulation interface {:?}",
            transfer
        );
    }

    /// Update the account
    fn update(&mut self, account: ServiceId) {
        unimplemented!(
            "make sure you are invoking the accumulation interface {:?}",
            account
        );
    }

    /// Upsert an account
    fn upsert(&mut self, id: ServiceId, account: impl Account) {
        unimplemented!(
            "make sure you are invoking the accumulation interface {:?}, {:?}",
            id,
            account.index()
        );
    }

    /// Read from memory
    fn read(&self, address: u32, len: u32) -> Result<Vec<u8>> {
        unimplemented!("make sure you are accessing a VM context: address={address} len={len}")
    }

    /// Read a hash from memory
    fn read_hash(&self, address: u32) -> Result<[u8; 32]> {
        let mut hash = [0; 32];
        hash.copy_from_slice(&self.read(address, 32)?);
        Ok(hash)
    }

    /// Write to memory
    fn write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        unimplemented!("make sure you are accessing a VM context: address={address} data={data:?}");
    }

    /// Allocate memory
    fn allocate(&mut self, start: u32, count: u32) -> Result<()> {
        unimplemented!("make sure you are accessing a VM context: start={start} count={count}")
    }

    /// Get the heap pointer
    fn heap_ptr(&self) -> u32 {
        unimplemented!("make sure you are accessing a VM context")
    }
}

impl Argument for () {
    const SUPPORTED_CALLS: &[u32] = &[];

    const INITIAL_PC: u64 = 0;

    fn account(&mut self, _id: u64) -> anyhow::Result<&mut impl Account> {
        anyhow::Result::<&mut ServiceAccount>::Err(anyhow::anyhow!("not implemented"))
    }

    fn this(&mut self) -> anyhow::Result<&mut impl Account> {
        anyhow::Result::<&mut ServiceAccount>::Err(anyhow::anyhow!("not implemented"))
    }
}
