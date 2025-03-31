//! The virtual machine interfaces of SpaceJam

use crate::{service::ServiceAccount, Gas, OpaqueHash, ServiceId, TimeSlot};
use std::collections::BTreeMap;
pub use {
    accumulate::{AccumulateResult, Operand},
    context::StateContext,
    transfer::DeferredTransfer,
};

mod accumulate;
mod context;
mod transfer;

/// Service to hash commitment map
pub type CommitmentMap = BTreeMap<ServiceId, OpaqueHash>;

/// The virtual machine interface
pub trait Vm {
    /// (Ψ): the general PVM invocation
    fn invoke(
        // (p) the program blob
        _blob: Vec<u8>,
        // (ı) the current program counter
        _pc: u64,
        // (ϱ) the gas
        _gas: Gas,
        // (ω) the registers
        _registers: [u64; 13],
        // (µ) the memory
        _memory: Vec<u32>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// (ΨA): single step state transition invocation
    ///
    /// as defined per graypaper (A.1)
    fn accumulate(
        // (U) The state context
        _context: StateContext,
        // (N_t)  timeslot for the current accumulation
        _slot: TimeSlot,
        // (N_s)  the service id of the caller
        _service_id: ServiceId,
        // (N_g)  the gas limit for the current operation
        _gas_limit: Gas,
        // (O)  the accumulation operands
        _operands: Vec<Operand>,
    ) -> AccumulateResult {
        Default::default()
    }

    /// (Ψ1): single step state transition invocation
    fn step(
        // (c) The instruction data
        _blob: Vec<u8>,
        // (k) The bitmap of the instruction data
        _bitmap: Vec<u8>,
        // (j) The jump table
        _jump_table: Vec<u64>,
        // (ı) The current program counter
        _pc: u64,
        // (ϱ) The gas
        _gas: Gas,
        // (ω) The registers
        _registers: [u64; 13],
        // (µ) The memory
        _memory: Vec<u32>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// (ΨT): on-transfer invocation
    fn transfer(
        // (δ) The account storage
        _accounts: &BTreeMap<ServiceId, ServiceAccount>,
        // (N_t)  timeslot for the current accumulation
        _slot: TimeSlot,
        // (N_s)  the service id of the caller
        _service_id: ServiceId,
        // (T)  the deferred transfers
        _transfers: &[DeferredTransfer],
    ) -> (ServiceAccount, Gas) {
        (ServiceAccount::default(), 0)
    }
}

impl Vm for () {}
