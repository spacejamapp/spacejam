//! PolkaVM implementation

pub use score::Gas;
pub use {
    invocation::Invocation,
    result::{Executed, Reason, Received, Refined, State, Stepped, Transfered},
    value::Value,
};

mod invocation;
pub mod program;
mod result;
mod value;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T> Pvm for T where T: Invocation {}
