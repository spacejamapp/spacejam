//! PolkaVM implementation

pub use score::Gas;
pub use {
    host::HostCall,
    invocation::Invocation,
    result::{Executed, Reason, Received, Refined, State, Stepped, Transferred},
    value::Value,
};

mod host;
mod invocation;
mod result;
mod value;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T> Pvm for T where T: Invocation {}
