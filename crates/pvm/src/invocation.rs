//! PVM invocation interface

use crate::{Executed, Reason, Received, Refined, State, Stepped, Transferred};
use pvm_parser::util;
use score::{
    service::{ServiceAccount, WorkExecResult, WorkPackage},
    vm::{AccumulateResult, DeferredTransfer, Operand, StateContext},
    Gas, ServiceId, TimeSlot,
};
use std::collections::BTreeMap;

/// The invocation interface of PVM
pub trait Invocation {
    /// The memory type of the PVM
    type Memory: Default + Clone;

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
        memory: Self::Memory,
    ) -> Stepped<Self::Memory, ()> {
        let mut state = State::<Self::Memory> {
            pc,
            gas: gas as i64,
            registers,
            memory,
        };

        // deblob the program
        let (instructions, bitmask, jump) = match util::deblob(blob) {
            Ok(program) => program,
            Err(e) => {
                return Stepped::new(Reason::Panic(e.to_string()), state);
            }
        };

        // stepping instructions
        loop {
            let Stepped {
                reason,
                state: next,
                data: _,
            } = Self::step(
                &instructions,
                &bitmask,
                &jump,
                state.pc,
                state.gas as u64,
                state.registers,
                state.memory.clone(),
            );

            // out of gas
            if next.gas < 0 {
                return Stepped::new(Reason::OOG, state);
            }

            // handle the exit reason
            state = next;
            match reason {
                // no exit reason, continue
                Reason::Continue => {
                    continue;
                }
                // reset the program counter on halt or panic
                Reason::Halt | Reason::Panic(_) => {
                    // TODO: stf and GP not matched
                    //
                    // state.pc = 0
                }
                _ => {}
            };

            return Stepped::new(reason, state);
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
        _memory: Self::Memory,
    ) -> Stepped<Self::Memory, ()>;

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
        _memory: Self::Memory,
        // (f) the host function
        _function: impl FnOnce(X) -> (Reason, State<Self::Memory>, X),
        // (x) the host function input data
        _input: X,
    ) -> Stepped<Self::Memory, X> {
        Stepped::new(Reason::Halt, State::<Self::Memory>::default())
    }

    /// (ΨM): argument invocation
    ///
    /// Defined per graypaper (A.43)
    fn argument<X: Default>(
        // (p) The program blob
        _blob: &[u8],
        // (ı) The current program counter
        _pc: u64,
        // (ϱ) The gas
        _gas: u64,
        // (a) The input data
        _input: &[u8],
        // (f) the host function
        _fun: impl FnOnce(X) -> (Reason, State<Self::Memory>, X),
        // (x) the host function input data
        _args: X,
    ) -> Received<X> {
        Received::new(0, Vec::new(), Reason::Halt)
    }

    /// (ΨI): The Is-Authorized invocation
    ///
    /// Defined per graypaper (B.1)
    fn is_authorized(
        // (p) The work package
        _package: WorkPackage,
        // (i) The core index
        _core_idx: usize,
    ) -> Executed {
        Executed::new(Vec::new(), WorkExecResult::Panic, 0)
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
    ) -> Refined {
        Refined::new(
            Executed::new(Vec::new(), WorkExecResult::Panic, 0),
            Vec::new(),
        )
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
    ) -> Transferred {
        Transferred::default()
    }
}

impl Invocation for () {
    type Memory = ();

    fn step(
        _instructions: &[u8],
        _bitmask: &[u8],
        _jump: &[u64],
        _pc: u64,
        _gas: Gas,
        _registers: [u64; 13],
        _memory: Self::Memory,
    ) -> Stepped<Self::Memory, ()> {
        Stepped::new(
            Reason::Panic("unimplemented".to_string()),
            State::<Self::Memory>::default(),
        )
    }
}
