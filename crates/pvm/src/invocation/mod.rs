//! PVM invocation interface

use crate::Reason;
use parser::program::{self, Program};
use score::{
    service::{WorkExecResult, WorkPackage},
    vm::{AccumulateParams, AccumulateState, DeferredTransfer, Operand, RefineParams},
    Account, Accounts, Gas, ServiceId, TimeSlot,
};
pub use {
    accumulate::{Accumulate, AccumulateContext, Accumulated},
    argument::Argument,
    authorize::IsAuthorized,
    general::General,
    refine::{Refine, Refined},
    state::{Executed, Received, State, Stepped},
    transfer::Transferred,
};

pub mod accumulate;
mod argument;
mod authorize;
mod general;
pub mod refine;
mod state;
pub mod transfer;

/// The invocation Interface of PVM
pub trait Invocation {
    /// Invoke a program with the given context (version 3)
    fn invoke2<X: Argument>(program: &Program, ctx: X, gas: Gas, pc: usize) -> Received<X> {
        let _ = (program, gas, pc);
        Received {
            gas: 0,
            output: vec![],
            reason: Reason::Panic("unimplemented".to_string()),
            data: ctx,
            state: Default::default(),
        }
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
        let Some(code) = account.historical_lookup(timeslot, package.auth_code_hash) else {
            tracing::warn!(
                "Authorization code not found for hash {:?}",
                package.auth_code_hash
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
        let program = program::preimage(&code, &args).expect("failed to preimage");
        let result = Self::invoke2(&program, context, score::GAS_IS_AUTHORIZED, 0);

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
        let refine = crate::invocation::Refine {
            accounts: accounts.clone(),
            service: item.service,
            core,
            auth_output: auth_output.to_vec(),
            all_imports: all_imports.to_vec(),
            export_offset,
            exports: Vec::new(),
        };

        let args = codec::encode(&params).expect("failed to encode params");
        let program = program::preimage(&code, &args).expect("failed to preimage");
        let result = Self::invoke2(&program, refine, item.refine_gas_limit, 5);

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
        let context = AccumulateContext::new(context, service, timeslot);
        let params = AccumulateParams {
            slot: timeslot,
            id: service,
            results: operands.len() as u32,
        };

        let accumulate = context.accumulate(timeslot, operands);
        let args = codec::encode(&params).expect("failed to encode");
        let program = program::preimage(&code, &args).expect("failed to preimage");
        let result = Self::invoke2(&program, accumulate, gas, 5);
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

        // Note: Balance update happens in defer_transfers, not here
        // according to Gray Paper the transfer invocation executes the recipient's
        // transfer handler but doesn't modify the balance directly
        let updated_account = account.account();
        let general = General::new(service, accounts, Vec::new(), Default::default());
        let input = codec::encode(&(slot, service, transfers)).expect("failed to encode");
        let program = program::preimage(&code, &input).expect("failed to preimage");
        let received = Self::invoke2(&program, general, gas, 10);
        Transferred {
            account: updated_account,
            gas: received.gas,
        }
    }

    /// (Ψ): the general PVM invocation
    ///
    /// defined per graypaper (A.1)
    #[deprecated(note = "non-production design from GP, use invoke2 instead")]
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
        memory: parser::Memory,
    ) -> Stepped<()> {
        let _ = (blob, pc, gas, registers, memory);
        Stepped::new(Reason::Panic("deprecated".to_string()), State::default())
    }

    /// (Ψ1): single-step state transition invocation
    ///
    /// Defined per graypaper (A.6)
    #[deprecated(note = "non-production design from GP, use invoke2 instead")]
    fn step(
        // (c) The instruction data
        instructions: &[u8],
        // (k) The bitmap of the instruction data
        bitmask: &[u8],
        // (j) The jump table
        jump: &[u64],
        // (ı) The current program counter
        pc: u64,
        // (ϱ) The gas
        gas: Gas,
        // (ω) The registers
        registers: [u64; 13],
        // (µ) The memory
        memory: parser::Memory,
    ) -> Stepped<()> {
        let _ = (instructions, bitmask, jump, pc, gas, registers, memory);
        Stepped::new(Reason::Panic("deprecated".to_string()), State::default())
    }

    /// (ΨH): host call invocation
    ///
    /// Defined per graypaper (A.34)
    #[deprecated(note = "non-production design from GP, use invoke2 instead")]
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
        memory: parser::Memory,
        // (f) the host function
        //
        // (x) the host function input data
        input: X,
    ) -> Stepped<X> {
        let _ = (code, pc, gas, registers, memory);
        Stepped::new(Reason::Panic("deprecated".to_string()), State::default()).with(input)
    }

    /// (ΨM): argument invocation
    ///
    /// Defined per graypaper (A.43)
    #[deprecated(note = "non-production design from GP, use invoke2 instead")]
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
        let _ = (blob, pc, gas, args);
        Received {
            gas: 0,
            output: vec![],
            reason: Reason::Panic("deprecated".to_string()),
            data,
            state: Default::default(),
        }
    }
}

impl Invocation for () {}
