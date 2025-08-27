//! Invocation context of the interpreter

use anyhow::Result;
use parser::MemoryLike;
use score::{
    safrole::ValidatorData,
    service::Privileges,
    vm::{DeferredTransfer, Operand},
    Account, Gas, OpaqueHash, ServiceId, TimeSlot, VALIDATORS_COUNT,
};
use std::{cell::RefCell, rc::Rc};
pub use {
    argument::Argument,
    state::{Executed, Received, State, Stepped},
};

mod argument;
mod state;

/// Helper context that wraps the invocation arguments and the memory.
pub struct Context<X: Argument, M: MemoryLike + Clone> {
    /// The context from the chain
    pub ctx: X,

    /// The registers of the context
    pub registers: [u64; 13],

    /// The gas of the context
    pub gas: i64,

    /// The hosting memory
    pub memory: Rc<RefCell<M>>,
}

impl<X: Argument, M: MemoryLike + Clone> Argument for Context<X, M> {
    const SUPPORTED_CALLS: &[u32] = X::SUPPORTED_CALLS;

    fn account(&mut self, id: u64) -> Result<&mut impl Account> {
        self.ctx.account(id)
    }

    fn burn(&mut self, gas: Gas) {
        self.gas -= gas as i64;
    }

    fn check(&mut self, index: ServiceId) -> ServiceId {
        self.ctx.check(index)
    }

    fn checkpoint(&mut self) {
        self.ctx.checkpoint()
    }

    fn entropy(&self) -> OpaqueHash {
        self.ctx.entropy()
    }

    fn gas(&self) -> Gas {
        self.gas as u64
    }

    fn index(&self) -> ServiceId {
        self.ctx.index()
    }

    fn operands(&self) -> &[Operand] {
        self.ctx.operands()
    }

    fn or_this(&mut self, account: u64) -> Result<&mut impl Account> {
        self.ctx.or_this(account)
    }

    fn output(&mut self, hash: OpaqueHash) {
        self.ctx.output(hash)
    }

    fn privileges(&self) -> Privileges {
        self.ctx.privileges()
    }

    fn rget(&mut self, reg: u8) -> u64 {
        self.registers[reg as usize]
    }

    fn rset(&mut self, reg: u8, value: u64) {
        self.registers[reg as usize] = value;
    }

    fn remove(&mut self, service: ServiceId) {
        self.ctx.remove(service)
    }

    fn service(&self) -> ServiceId {
        self.ctx.service()
    }

    fn set_index(&mut self, index: ServiceId) {
        self.ctx.set_index(index)
    }

    fn set_authorization(&mut self, core: u16, queue: Vec<[u8; 32]>) {
        self.ctx.set_authorization(core, queue)
    }

    fn set_assign(&mut self, core: u16, assign: ServiceId) {
        self.ctx.set_assign(core, assign)
    }

    fn set_privileges(&mut self, privileges: Privileges) {
        self.ctx.set_privileges(privileges)
    }

    fn set_validators(&mut self, validators: [ValidatorData; VALIDATORS_COUNT as usize]) {
        self.ctx.set_validators(validators)
    }

    fn this(&mut self) -> Result<&mut impl Account> {
        self.ctx.this()
    }

    fn timeslot(&self) -> TimeSlot {
        self.ctx.timeslot()
    }

    fn transfer(&mut self, transfer: DeferredTransfer) {
        self.ctx.transfer(transfer)
    }

    fn update(&mut self, account: ServiceId) {
        self.ctx.update(account)
    }

    fn upsert(&mut self, id: ServiceId, account: impl Account) {
        self.ctx.upsert(id, account)
    }

    fn read(&self, address: u32, len: u32) -> Result<Vec<u8>> {
        self.memory.borrow().read(address, len)
    }

    fn write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        self.memory.borrow_mut().write(address, data)
    }

    fn allocate(&mut self, start: u32, count: u32) -> Result<()> {
        self.memory.borrow_mut().allocate(start, count)
    }

    fn heap_ptr(&self) -> u32 {
        self.memory.borrow().heap_ptr()
    }
}
