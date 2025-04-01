//! PolkaVM implementation

pub use {
    invocation::Invocation,
    result::{Reason, Result, State},
};

mod invocation;
pub mod program;
mod result;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T: Invocation> Pvm for T {}
