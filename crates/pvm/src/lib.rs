//! PolkaVM implementation

pub use score::Gas;
pub use {
    invocation::Invocation,
    memory::{Access, Memory},
    result::{Executed, Reason, Received, Refined, State, Stepped, Transfered},
};

mod invocation;
mod memory;
pub mod program;
mod result;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T: Invocation> Pvm for T {}
