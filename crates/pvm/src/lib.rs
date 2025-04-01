//! PolkaVM implementation

pub use invocation::Invocation;

mod invocation;

/// The PVM interface
pub trait Pvm: Invocation {}

impl<T: Invocation> Pvm for T {}
