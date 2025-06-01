//! Result type for the interpreter

use pvm::Reason;

/// The error type for the interpreter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The memory is inaccessible.
    MemoryInaccessible(u32),

    /// The memory is immutable.
    MemoryImmutable(u32),

    /// The jump to halt.
    Terminate,

    /// The dynamic jump is invalid.
    InvalidDynamicJump,

    /// Out of gas.
    OOG,

    /// The trap instruction was executed.
    Trap(bool),

    /// Host call.
    HostCall(u32),
}

impl Error {
    /// Get the extra gas for the error.
    pub fn extra_gas(&self) -> u64 {
        match self {
            Error::MemoryInaccessible(_) => 1,
            Error::MemoryImmutable(_) => 1,
            Error::Trap(true) => 1,
            Error::OOG => 0,
            _ => 0,
        }
    }
}

/// Convert an error to a reason.
impl From<Error> for Reason {
    fn from(error: Error) -> Self {
        match error {
            Error::MemoryInaccessible(address) => Reason::Fault {
                page: address / crate::PAGE_SIZE,
            },
            Error::MemoryImmutable(page) => Reason::Fault { page },
            Error::Terminate => Reason::Halt,
            Error::InvalidDynamicJump => Reason::Panic("invalid dynamic jump".into()),
            Error::Trap(_) => Reason::Panic("trap".into()),
            Error::OOG => Reason::OOG,
            Error::HostCall(call) => Reason::HostCall(call),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// The result type for the interpreter.
pub type Result<T> = std::result::Result<T, Error>;
