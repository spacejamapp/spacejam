//! PolkaVM implementation

pub use score::Gas;
pub use {
    host::Argument,
    invocation::Invocation,
    memory::Memory,
    result::{Executed, Reason, Received, Refined, Result, State, Stepped, Transferred},
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
