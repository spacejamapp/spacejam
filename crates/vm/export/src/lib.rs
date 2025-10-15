//! SpaceVM exports

use pvm::{AccumulateState, Invocation, Reason, codec, score, score::safrole::ValidatorData};
use score::{
    svc::api::{self, AccumulateArgs, Accumulated, AuthorizeArgs, RefineArgs},
    vm::AccumulateItems,
};

mod comp;
mod interp;

/// Initialize the logger
#[unsafe(no_mangle)]
pub extern "C" fn init_logger(ansi: bool, timer: bool) {
    let builder = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(ansi);

    if !timer {
        builder.without_time().init()
    } else {
        builder.init()
    }
}

/// (ΨI): Is-Authorized invocation
pub fn authorize<VM: Invocation>(buffer: Buffer) -> Buffer {
    let mut args: AuthorizeArgs = codec::decode(buffer.as_slice()).unwrap();
    let output = VM::is_authorized(
        &args.package,
        args.core_idx,
        &mut args.accounts,
        args.timeslot,
    );
    let output = codec::encode(&output).unwrap();
    Buffer::from(output)
}

/// (ΨR): Refine invocation
pub fn refine<VM: Invocation>(args: Buffer) -> Buffer {
    let mut args: RefineArgs = codec::decode(args.as_slice()).unwrap();
    let all_imports = args
        .all_imports
        .iter()
        .map(|s| s.iter().map(|s| s.0).collect())
        .collect::<Vec<Vec<[u8; 4104]>>>();
    let output = VM::refine(
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
    Buffer::from(output)
}

/// (ΨA): Accumulation invocation
pub fn accumulate<VM: Invocation>(args: Buffer) -> Buffer {
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

    // TODO: handle accumulate items
    let accumulated = VM::accumulate(
        context,
        args.timeslot,
        args.service,
        args.gas,
        AccumulateItems {
            operands: args.operands,
            transfers: vec![],
        },
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
    Buffer::from(output)
}

/// A buffer that host args / results
#[repr(C)]
#[derive(Debug)]
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

impl From<Vec<u8>> for Buffer {
    fn from(value: Vec<u8>) -> Self {
        let layout = std::alloc::Layout::from_size_align(value.len(), 1).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr(), ptr, value.len());
        }
        Buffer {
            ptr,
            len: value.len(),
        }
    }
}
