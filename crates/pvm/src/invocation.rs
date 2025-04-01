//! PVM invocation interface

use crate::{program, Reason, State};
use score::{
    service::{ServiceAccount, WorkExecResult, WorkPackage},
    vm::{AccumulateResult, DeferredTransfer, Operand, StateContext},
    Gas, ServiceId, TimeSlot,
};
use std::collections::BTreeMap;

/// The invocation interface of PVM
pub trait Invocation {
    /// (Ψ): the general PVM invocation
    ///
    /// defined per graypaper (A.1)
    fn invoke(
        // (p) the program blob
        blob: &[u8],
        // (ı) the current program counter
        pc: u64,
        // (ϱ) the gas
        gas: Gas,
        // (ω) the registers
        registers: [u64; 13],
        // (µ) the memory
        memory: Vec<u32>,
    ) -> (Reason, State) {
        let mut state = State {
            pc,
            gas: gas as i64,
            registers,
            memory,
        };

        // deblob the program
        let (instructions, bitmask, jump) = match program::deblob(blob) {
            Ok(program) => program,
            Err(e) => return (Reason::Panic(e.to_string()), state),
        };

        // stepping instructions
        loop {
            let (reason, next) = Self::step(
                &instructions,
                &bitmask,
                &jump,
                state.pc,
                state.gas as u64,
                state.registers,
                state.memory.clone(),
            );

            // out of gas
            if state.gas < 0 {
                return (Reason::OOG, state);
            }

            // handle the exit reason
            match reason {
                // no exit reason, continue
                Reason::Continue => {
                    state = next;
                    continue;
                }
                // reset the program counter on halt or panic
                Reason::Halt | Reason::Panic(_) => state.pc = 0,
                _ => {}
            };

            return (reason, state);
        }
    }

    /// (Ψ1): single-step state transition invocation
    ///
    /// Defined per graypaper (A.6)
    fn step(
        // (c) The instruction data
        _instructions: &[u8],
        // (k) The bitmap of the instruction data
        _bitmask: &[u8],
        // (j) The jump table
        _jump: &[u64],
        // (ı) The current program counter
        _pc: u64,
        // (ϱ) The gas
        _gas: Gas,
        // (ω) The registers
        _registers: [u64; 13],
        // (µ) The memory
        _memory: Vec<u32>,
    ) -> (Reason, State);

    /// (ΨH): host call invocation
    ///
    /// Defined per graypaper (A.34)
    fn call<X: Default>(
        // (c) The instruction data
        _instructions: &[u8],
        // (ı) The current program counter
        _pc: u64,
        // (ϱ) The gas
        _gas: u64,
        // (ω) The registers
        _registers: [u64; 13],
        // (µ) The memory
        _memory: Vec<u32>,
        // Ω⟨X⟩ the host function
        _function: impl FnOnce(X) -> (Reason, State, X),
        // X the host function input data
        _input: X,
    ) -> (Reason, State, X) {
        (Reason::Halt, State::default(), X::default())
    }

    /// (ΨM): argument invocation
    ///
    /// Defined per graypaper (A.43)
    fn argument<X: Default>(
        _blob: &[u8],
        _pc: u64,
        _gas: u64,
        _input: &[u8],
        _fun: impl FnOnce(X) -> (Reason, State, X),
        _args: X,
    ) -> (Gas, (Vec<u8>, Reason, X)) {
        (0, (Vec::new(), Reason::Halt, X::default()))
    }

    /// (ΨI): The Is-Authorized invocation
    ///
    /// Defined per graypaper (B.1)
    fn is_authorized(
        // (p) The work package
        _package: WorkPackage,
        // (i) The core index
        _core_idx: usize,
    ) -> ((Vec<u8>, WorkExecResult), Gas) {
        ((Vec::new(), WorkExecResult::Panic), 0)
    }

    // TODO: complete the signature
    /// (ΨR): Refine invocation
    ///
    /// Defined per graypaper (B.5)
    fn refine(
        // (i) the index of the work item to refine
        _work_idx: usize,
        // (p) the work package
        _package: WorkPackage,
        // (o) the authorizer output
        _output: Vec<u8>,
        // (i) import segments
        _imports: Vec<Vec<[u8; score::SEGMENT_SIZE]>>,
        // (ς) export segment offset
        _export_offset: usize,
    ) -> (
        (Vec<u8>, WorkExecResult),
        Vec<[u8; score::SEGMENT_SIZE]>,
        Gas,
    ) {
        ((Vec::new(), WorkExecResult::Panic), Vec::new(), 0)
    }

    /// (ΨA): Accumulation invocation
    ///
    /// as defined per graypaper (B.9)
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

    /// (ΨT): on-transfer invocation
    ///
    /// Defined per graypaper (B.15)
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

impl Invocation for () {
    fn step(
        _instructions: &[u8],
        _bitmask: &[u8],
        _jump: &[u64],
        _pc: u64,
        _gas: Gas,
        _registers: [u64; 13],
        _memory: Vec<u32>,
    ) -> (Reason, State) {
        (Reason::Continue, State::default())
    }
}
