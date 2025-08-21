//! PolkaVM implementation

pub use parser::Memory;
pub use score::{Account, Accounts, Gas};
pub use {
    invocation::{
        accumulate::{AccumulateContext, Accumulated},
        refine::Refined,
        transfer::Transferred,
        Argument, Executed, Invocation, Received, State, Stepped,
    },
    result::{Reason, Result},
    value::Value,
};

/// Bail out with a panic
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::Reason::Panic(format!($($arg)*)))
    };
}

pub mod host;
mod invocation;
mod result;
mod value;

/// (Z_A) The alignment factor of the jump table.
pub const JUMP_ALIGNMENT_FACTOR: u32 = 2;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T> Pvm for T where T: Invocation {}
