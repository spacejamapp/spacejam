//! Extrinsic Result

use core::fmt::Display;

/// Extrinsic validation error
#[derive(Debug)]
pub enum Error {
    /// Extrinsic is validated
    ExtrinsicValidated,

    Validation(Box<dyn ValidationError + Send>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

/// Error happens in validation
pub trait ValidationError: std::error::Error + Display {}

/// Extrinsic validation result
pub type Result<T> = std::result::Result<T, Error>;
