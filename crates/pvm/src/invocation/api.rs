//! API for the invocation

use crate::{
    host,
    invocation::{General, Received, State, Stepped},
    AccumulateContext, Accumulated, Argument, Executed, Memory as _, Reason, Refined, Transferred,
};
use parser::{
    program::{self, Program},
    ProgramBlob,
};
use score::{
    service::{WorkExecResult, WorkPackage},
    vm::{AccumulateParams, AccumulateState, DeferredTransfer, Operand, RefineParams},
    Account, Accounts, Gas, OpaqueHash, ServiceId, TimeSlot,
};

/// The invocation Interface of PVM
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
                return state.stepped(Reason::Panic(e.to_string()));
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
                return state.stepped(Reason::OOG);
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

            return state.stepped(reason);
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
    fn call<R: Accounts, X: Argument<R>>(
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
            data: _,
        } = Self::invoke(code, pc, gas, registers, memory);
        let Reason::HostCall(call) = reason else {
            return state.stepped(reason).with(input);
        };

        // FIXME: refactor with loop
        let stepped = host::call::<R, _, _>(call, state, input);
        match stepped.reason {
            Reason::Fault { page } => stepped
                .state
                .stepped(Reason::Fault { page })
                .with(stepped.data),
            Reason::Continue | Reason::HostCall(_) => Self::call(
                code,
                stepped.state.pc,
                stepped.state.gas as u64,
                stepped.state.registers,
                stepped.state.memory,
                stepped.data,
            ),
            _ => stepped.state.stepped(stepped.reason).with(stepped.data),
        }
    }

    /// (ΨM): argument invocation
    ///
    /// Defined per graypaper (A.43)
    fn argument<R: Accounts, X: Argument<R>>(
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
                return Received::panic(e, data);
            }
        };

        let memory = Self::Memory::from_raw(memory);
        let mut stepped = Self::call(&code, pc, gas, registers, memory, data);

        // get the output
        let mut output = vec![];
        if stepped.reason == Reason::Halt {
            let ptr = stepped.state.registers[7] as u32;
            let len = stepped.state.registers[8] as u32;

            // Read output data from memory using ptr and len
            if let Ok(data) = stepped.state.memory.read_bytes(ptr, len) {
                output = data;
            } else {
                stepped.reason = Reason::Panic("failed to read output from memory".to_string());
            }
        }

        let gas = gas - (stepped.state.gas.max(0) as u64);
        stepped.received(gas, output)
    }

    /// (ΨI): The Is-Authorized invocation
    ///
    /// Defined per graypaper (B.5)
    fn is_authorized<R: Accounts>(
        // (p) the work package
        package: &WorkPackage,
        // (c) The core index
        core_idx: u16,
        // (δ) accounts for historical lookup
        accounts: &mut R,
        // (N_t) timeslot for the current operation
        timeslot: TimeSlot,
    ) -> Executed {
        // Get the service account that hosts the authorization code
        let Some(account) = accounts.get(package.auth_code_host) else {
            tracing::warn!(
                "Authorization code host service {} not found",
                package.auth_code_host
            );
            return Executed::new(Vec::new(), WorkExecResult::BadCode, 0);
        };

        // Resolve authorization code using historical lookup
        let Some(code) = account.historical_lookup(timeslot, package.authorizer.code_hash) else {
            tracing::warn!(
                "Authorization code not found for hash {:?}",
                package.authorizer.code_hash
            );
            return Executed::new(Vec::new(), WorkExecResult::BadCode, 0);
        };

        // Check authorization code size limit (W_A - BIG if too big)
        if code.len() > score::MAX_IS_AUTHORIZED_CODE_SIZE as usize {
            tracing::warn!(
                "Authorization code too big: {} bytes > {} bytes limit",
                code.len(),
                score::MAX_IS_AUTHORIZED_CODE_SIZE as usize
            );
            return Executed::new(Vec::new(), WorkExecResult::CodeOversize, 0);
        }

        // Prepare arguments
        let args = codec::encode(&core_idx).unwrap_or_default();
        let context = crate::invocation::IsAuthorized::new(package.clone(), core_idx);
        let result = Self::argument::<R, _>(&code, 0, score::GAS_IS_AUTHORIZED, &args, context);

        // construct the result
        let gas = result.gas;
        let output = result.output.clone();
        let exec_result = result.result();
        Executed::new(output, exec_result, gas)
    }

    /// (ΨR): Refine invocation
    ///
    /// Defined per graypaper (B.5)
    #[allow(clippy::too_many_arguments)]
    fn refine<R: Accounts>(
        // (c) the core index
        core: u16,
        // (i) the work item index
        index: usize,
        // (p) the work package
        package: &WorkPackage,
        // (r) the authorizer output
        auth_output: &[u8],
        // (ī) all work items' import segments
        all_imports: &[Vec<[u8; score::SEGMENT_SIZE as usize]>],
        // (ς) export segment offset
        export_offset: u16,
        // (δ) accounts for historical lookup
        accounts: &mut R,
        // (N_t) timeslot for the current operation
        timeslot: TimeSlot,
    ) -> Refined {
        let item = &package.items[index];
        let Some(account) = accounts.get(item.service) else {
            tracing::warn!("no account found for service: {}", item.service);
            return Refined::new(
                Executed::new(Vec::new(), WorkExecResult::BadCode, 0),
                Vec::new(),
            );
        };

        let Some(code) = account.historical_lookup(timeslot, item.code_hash) else {
            tracing::warn!("no code found for service: {}", item.service);
            return Refined::new(
                Executed::new(Vec::new(), WorkExecResult::BadCode, 0),
                Vec::new(),
            );
        };

        if code.len() > score::MAX_REFINE_CODE_SIZE as usize {
            return Refined::new(
                Executed::new(Vec::new(), WorkExecResult::CodeOversize, 0),
                Vec::new(),
            );
        }

        // FIXME: passing the hash into this function mb. do not hash it for twice!
        let package_hash =
            crypto::blake2b(&codec::encode(package).expect("failed to encode package"));
        let params = RefineParams {
            core,
            index: index as u16,
            id: item.service,
            payload: item.payload.clone(),
            package: package_hash,
        };

        // Get import segments for this work item
        let _work_item_imports = if index < all_imports.len() {
            all_imports[index].clone()
        } else {
            Vec::new()
        };

        // Create refine context with proper parameters
        let refine_context = crate::invocation::Refine {
            accounts: accounts.clone(),
            service: item.service,
            core,
            auth_output: auth_output.to_vec(),
            all_imports: all_imports.to_vec(),
            export_offset,
            exports: Vec::new(),
        };

        let result = Self::argument::<R, _>(
            &code,
            0,
            item.refine_gas_limit,
            &codec::encode(&params).expect("failed to encode params"),
            refine_context,
        );

        // TODO: Implement actual segment export when host calls are ready
        // For now, return empty segments as before
        let gas = result.gas;
        Refined::new(Executed::new(Vec::new(), result.result(), gas), Vec::new())
    }

    /// (ΨA): Accumulation invocation
    ///
    /// as defined per graypaper (B.9)
    fn accumulate<R: Accounts>(
        // (U) The state context
        mut context: AccumulateState<R>,
        // (N_t)  timeslot for the current accumulation
        timeslot: TimeSlot,
        // (N_s)  the service id of the caller
        service: ServiceId,
        // (N_g)  the gas limit for the current operation
        gas: Gas,
        // (O)  the accumulation operands
        operands: Vec<Operand>,
    ) -> Accumulated<R> {
        let Some(code) = context.code(service) else {
            tracing::warn!("no code found for service: {}", service);
            return Accumulated::new(context);
        };

        // create the accumulate context
        let entropy = context.entropy[0];
        let context = AccumulateContext {
            context,
            service,
            index: Self::index(service, timeslot, entropy),
            transfer: Vec::new(),
            output: None,
        };

        let params = AccumulateParams {
            slot: timeslot,
            id: service,
            results: operands.len() as u32,
        };

        let accumulate = context.accumulate(timeslot, operands);
        let args = codec::encode(&params).expect("failed to encode");
        let result = Self::argument(&code, 5, gas, &args, accumulate);
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
    fn transfer<R: Accounts>(
        // (δ) The account storage
        mut accounts: R,
        // (N_t)  timeslot for the current accumulation
        slot: TimeSlot,
        // (N_s)  the service id of the caller
        service: ServiceId,
        // (T)  the deferred transfers
        transfers: &[DeferredTransfer],
    ) -> Transferred {
        let Some(account) = accounts.get(service) else {
            tracing::warn!("no account found for service: {}", service);
            return Transferred::default();
        };

        let Some(code) = account.blob() else {
            return Transferred::default();
        };

        let code = code.clone();
        let gas = transfers.iter().map(|t| t.gas_limit).sum::<Gas>();
        let amount = transfers.iter().map(|t| t.amount).sum::<u64>();

        // TODO: update the account balance ???
        //
        // this seems not correct.
        tracing::warn!("FIXME: update the account balance: {}", amount);
        *account.balance_mut() += amount;
        let account = account.account();
        let general = General::new(service, accounts, Vec::new(), Default::default());
        let input = codec::encode(&(slot, service, transfers)).expect("failed to encode");
        let received = Self::argument(&code, 10, gas, &input, general);
        Transferred {
            account,
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
