//! PVM invocation interface

use crate::{
    host, AccumulateContext, AccumulateResult, Argument, Executed, Memory as _, Reason, Received,
    Refined, State, Stepped, Transferred,
};
use parser::{
    program::{self, Program},
    ProgramBlob,
};
use score::{
    service::{ServiceAccount, WorkExecResult, WorkPackage},
    vm::{AccumulateParams, DeferredTransfer, Operand, StateContext},
    Gas, OpaqueHash, ServiceId, TimeSlot,
};
use std::collections::BTreeMap;

/// The invocation interface of PVM
///
/// TODO: refactor this interface when the implementation gets stable.
pub trait Invocation {
    /// The memory type of the PVM
    type Memory: crate::Memory;

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
        let ProgramBlob {
            instructions,
            bitmask,
            jump_table: jump,
        } = match program::deblob(blob) {
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
                Reason::Halt | Reason::Panic(_) => {}
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
    fn call<X: Argument>(
        // (c) The instruction data
        code: &[u8],
        // (ı) The current program counter
        pc: u64,
        // (ϱ) The gas
        gas: u64,
        // (ω) The registers
        registers: [u64; 13],
        // (µ) The memory
        memory: Self::Memory,
        // (f) the host function
        //
        // (x) the host function input data
        input: X,
    ) -> Stepped<Self::Memory, X> {
        // (state') invoke the PVM
        let Stepped {
            reason,
            state,
            data: _, // invoke() returns () data, but we need to preserve input
        } = Self::invoke(code, pc, gas, registers, memory);

        // if error occurs, return the state WITH THE PRESERVED INPUT DATA
        let Reason::HostCall(call) = reason else {
            return Stepped::new(reason, state).with(input);
        };

        let stepped = host::call(call, state, input);
        match stepped.reason {
            Reason::Fault { page } => {
                Stepped::new(Reason::Fault { page }, stepped.state).with(stepped.data)
            }
            // TODO: this recursive call should be optimized in production.
            //
            // mb create a new call_inner function and set up a loop for it.
            Reason::Continue | Reason::HostCall(_) => Self::call(
                code,
                stepped.state.pc,
                stepped.state.gas as u64,
                stepped.state.registers,
                stepped.state.memory,
                stepped.data,
            ),
            _ => Stepped::new(stepped.reason, stepped.state).with(stepped.data),
        }
    }

    /// (ΨM): argument invocation
    ///
    /// Defined per graypaper (A.43)
    fn argument<X: Argument>(
        // (p) The standard program blob
        blob: &[u8],
        // (ı) The current program counter
        pc: u64,
        // (ϱ) The gas
        gas: u64,
        // (a) The input data
        args: &[u8],
        // (f) the host function
        //
        // (x) the host function input data
        data: X,
    ) -> Received<X> {
        let Program {
            code,
            registers,
            memory,
        } = match program::preimage(blob, args) {
            Ok(standard) => standard,
            Err(e) => {
                tracing::error!("failed to deblob the standard program blob: {e:?}");
                return Received::new(0, Reason::Panic(e.to_string()), data);
            }
        };

        let memory = Self::Memory::from_raw(memory);
        let stepped = Self::call(&code, pc, gas, registers, memory, data);

        // get the output
        let mut output = vec![];
        let registers = stepped.state.registers;
        let registered = [registers[7].to_le_bytes(), registers[8].to_le_bytes()].concat();
        if stepped.reason == Reason::Halt && stepped.state.memory.contains(&registered) {
            output = registered;
        };

        Received::new(
            gas - (stepped.state.gas.max(0) as u64),
            stepped.reason,
            stepped.data,
        )
        .with(output)
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
        context: StateContext,
        // (N_t)  timeslot for the current accumulation
        timeslot: TimeSlot,
        // (N_s)  the service id of the caller
        service: ServiceId,
        // (N_g)  the gas limit for the current operation
        gas: Gas,
        // (O)  the accumulation operands
        operands: Vec<Operand>,
        // entropy'0
        entropy: OpaqueHash,
    ) -> AccumulateResult {
        let Some(code) = context.code(service) else {
            tracing::trace!("no code found for service: {}", service);
            return AccumulateResult {
                context,
                ..Default::default()
            };
        };

        // create the accumulate context
        let context = AccumulateContext {
            context: context.clone(),
            service,
            index: Self::index(service, timeslot, entropy),
            transfer: Vec::new(),
            output: None,
        };

        let accumulate = host::Accumulate::new(context, timeslot);
        let params = AccumulateParams {
            slot: timeslot,
            id: service,
            results: operands,
        };
        tracing::debug!("accumulate params: {:?}", params);
        let args = codec::encode(&params).expect("failed to encode");
        let result = Self::argument(code, 5, gas, &args, accumulate);
        if result.reason != Reason::Continue && result.reason != Reason::Halt {
            tracing::warn!(
                "PVM execution stopped with reason: {:?} for service {}",
                result.reason,
                service
            );
        } else {
            tracing::debug!(
                "PVM execution continued for service {}, reason: {:?}",
                service,
                result.reason
            );
        }

        result.to_result()
    }

    /// (ΨT): on-transfer invocation
    ///
    /// Defined per graypaper (B.15)
    fn transfer(
        // (δ) The account storage
        accounts: &BTreeMap<ServiceId, ServiceAccount>,
        // (N_t)  timeslot for the current accumulation
        slot: TimeSlot,
        // (N_s)  the service id of the caller
        service: ServiceId,
        // (T)  the deferred transfers
        transfers: &[DeferredTransfer],
    ) -> Transferred {
        let Some(account) = accounts.get(&service) else {
            tracing::warn!("no account found for service: {}", service);
            return Transferred::default();
        };

        let Some(code) = account.code() else {
            return Transferred::default();
        };

        let code = code.clone();
        let gas = transfers.iter().map(|t| t.gas_limit).sum::<Gas>();
        let amount = transfers.iter().map(|t| t.amount).sum::<u64>();

        // TODO: update the account balance ???
        //
        // this seems not correct.
        tracing::warn!("FIXME: update the account balance: {}", amount);
        let mut account = account.clone();
        account.balance += amount;
        let general = host::General {
            account,
            index: service,
            accounts: accounts.clone(),
        };

        let input = codec::encode(&(slot, service, transfers)).expect("failed to encode");
        let received = Self::argument(&code, 10, gas, &input, general);
        Transferred {
            account: received.data.account,
            gas: received.gas,
        }
    }

    /// (I) Generate a new index from provided environment
    fn index(service: ServiceId, timeslot: TimeSlot, entropy: OpaqueHash) -> ServiceId {
        let encoded = codec::encode(&(service, entropy, timeslot)).expect("failed to encode");
        let hash = crypto::blake2b(&encoded);
        let mut lebytes = [0; 4];
        lebytes[0..4].copy_from_slice(&hash[0..4]);

        let base = u32::from_le_bytes(lebytes);
        base % (u32::MAX - (1 << 9)) + (1 << 8)
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
