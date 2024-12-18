//! Result type for the PVM

use crate::status::Status;

/// The execution result of the interpreter.
pub type Result<T> = core::result::Result<T, Error>;

/// The error type for the interpreter.
#[derive(Debug)]
pub enum Error {
    /// The error is a status.
    Status(Status),

    /// The error is an anyhow error.
    Anyhow(anyhow::Error),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Anyhow(err)
    }
}

impl From<Status> for Error {
    fn from(status: Status) -> Self {
        Self::Status(status)
    }
}
