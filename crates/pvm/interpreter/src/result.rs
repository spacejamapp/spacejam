//! Result type for the interpreter

use crate::Status;

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

    /// The trap instruction was executed.
    Trap(bool),
}

impl Error {
    /// Get the extra gas for the error.
    pub fn extra_gas(&self) -> u32 {
        match self {
            Error::MemoryInaccessible(_) => 1,
            Error::MemoryImmutable(_) => 1,
            Error::Trap(true) => 1,
            _ => 0,
        }
    }
}

/// Convert an error to a status.
impl From<Error> for Status {
    fn from(error: Error) -> Self {
        match error {
            Error::MemoryInaccessible(address) => Status::Fault(address),
            Error::MemoryImmutable(address) => Status::Fault(address),
            Error::Terminate => Status::Halt,
            Error::InvalidDynamicJump => Status::Panic,
            Error::Trap(_) => Status::Panic,
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
