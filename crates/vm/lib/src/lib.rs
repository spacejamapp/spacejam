//! SpaceVM common library

use service::api::{self, AccumulateArgs, Accumulated, AuthorizeArgs, RefineArgs};
use spacevm::{
    SpaceVM,
    pvm::{AccumulateState, Invocation, Reason, score::safrole::ValidatorData},
};

/// (ΨA): Accumulation invocation
#[unsafe(no_mangle)]
pub extern "C" fn accumulate(args: Buffer) -> Buffer {
    let args: AccumulateArgs = codec::decode(args.as_slice()).unwrap();
    let context = AccumulateState {
        accounts: args.context.accounts,
        validators: args.context.validators.map(|v| ValidatorData {
            bandersnatch: v.bandersnatch,
            ed25519: v.ed25519,
            bls: v.bls,
            metadata: v.metadata,
        }),
        authorization: args.context.authorization.clone(),
        privileges: args.context.privileges,
        entropy: args.context.entropy,
    };
    let accumulated = <SpaceVM as Invocation>::accumulate(
        context,
        args.timeslot,
        args.service,
        args.gas,
        args.operands,
    );

    let output = Accumulated {
        context: api::AccumulateState {
            accounts: accumulated.context.accounts,
            validators: accumulated.context.validators.map(|v| api::ValidatorData {
                bandersnatch: v.bandersnatch,
                ed25519: v.ed25519,
                bls: v.bls,
                metadata: v.metadata,
            }),
            authorization: accumulated.context.authorization,
            privileges: accumulated.context.privileges,
            entropy: accumulated.context.entropy,
        },
        transfers: accumulated.transfers,
        hash: accumulated.hash,
        gas: accumulated.gas,
        reason: match accumulated.reason {
            Reason::Halt => api::Reason::Halt,
            Reason::Panic(message) => api::Reason::Panic(message),
            Reason::Fault { page } => api::Reason::Fault { page },
            Reason::HostCall(addr) => api::Reason::HostCall(addr),
            Reason::OOG => api::Reason::OOG,
            Reason::Continue => api::Reason::Continue,
        },
    };
    let output = codec::encode(&output).unwrap();
    Buffer {
        ptr: output.as_ptr(),
        len: output.len(),
    }
}

/// (ΨR): Refine invocation
#[unsafe(no_mangle)]
pub extern "C" fn refine(args: Buffer) -> Buffer {
    let mut args: RefineArgs = codec::decode(args.as_slice()).unwrap();
    let all_imports = args
        .all_imports
        .iter()
        .map(|s| s.iter().map(|s| s.0).collect())
        .collect::<Vec<Vec<[u8; 4104]>>>();
    let output = <SpaceVM as Invocation>::refine(
        args.core,
        args.index,
        &args.package,
        &args.auth_output,
        &all_imports,
        args.export_offset,
        &mut args.accounts,
        args.timeslot,
    );
    let output = codec::encode(&output).unwrap();
    Buffer {
        ptr: output.as_ptr(),
        len: output.len(),
    }
}

/// (ΨI): Is-Authorized invocation
#[unsafe(no_mangle)]
pub extern "C" fn authorize(buffer: Buffer) -> Buffer {
    let mut args: AuthorizeArgs = codec::decode(buffer.as_slice()).unwrap();
    let output = <SpaceVM as Invocation>::is_authorized(
        &args.package,
        args.core_idx,
        &mut args.accounts,
        args.timeslot,
    );
    let output = codec::encode(&output).unwrap();
    Buffer {
        ptr: output.as_ptr(),
        len: output.len(),
    }
}

/// A buffer that host args / results
#[repr(C)]
pub struct Buffer {
    /// The pointer to the buffer
    pub ptr: *const u8,
    /// The length of the buffer
    pub len: usize,
}

impl Buffer {
    /// Get the buffer as a byte slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}
