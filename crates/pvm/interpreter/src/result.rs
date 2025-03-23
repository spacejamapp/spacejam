//! Result type for the interpreter

use crate::Status;

/// The error type for the interpreter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The memory is inaccessible.
    MemoryInaccessible,

    /// The memory is immutable.
    MemoryImmutable,
}

/// Convert an error to a status.
impl From<Error> for Status {
    fn from(error: Error) -> Self {
        match error {
            Error::MemoryInaccessible => Status::Fault,
            Error::MemoryImmutable => Status::Fault,
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
