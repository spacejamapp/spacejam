//! PolkaVM implementation

pub use score::{account::Accounts, Gas};
pub use {
    host::Argument,
    invocation::{
        accumulate::{AccumulateContext, AccumulateResult},
        refine::Refined,
        transfer::Transferred,
        Executed, Invocation, Received, State, Stepped,
    },
    memory::Memory,
    result::{Reason, Result},
    value::Value,
};

/// Bail out with a panic
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err(Reason::Panic(format!($($arg)*)))
    };
}

pub mod host;
mod invocation;
mod memory;
mod result;
mod value;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T> Pvm for T where T: Invocation {}
