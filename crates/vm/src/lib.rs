//! PolkaVM implementation

pub use parser::{
    program::Program, Memory, MemoryInfo, Visitor, PAGE_SIZE, PVM_MEMORY_SIZE, ZONE_SIZE,
};
pub use {
    account::Account,
    codec,
    context::{check_range, Argument, Context, Executed, Invoked, MemoryLike, State},
    invocation::{AccumulateContext, AccumulateState, Accumulated, Invocation},
    parser,
    result::{Reason, Result},
    score,
    value::{as_i64, Value},
};

/// Bail out with a panic
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::Reason::Panic(format!($($arg)*)))
    };
}

mod context;
pub mod host;
pub mod invocation;
mod result;
mod value;

/// (Z_A) The alignment factor of the jump table.
pub const JUMP_ALIGNMENT_FACTOR: u32 = 2;

/// The number of registers.
pub const REGISTER_COUNT: usize = 13;

/// The maximum number of functions.
pub const MAX_FUNCTIONS: usize = 512;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T> Pvm for T where T: Invocation {}
