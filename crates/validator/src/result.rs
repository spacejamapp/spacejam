//! Extrinsic Result

/// Extrinsic validation error
pub enum Error {
    /// Extrinsic is validated
    ExtrinsicValidated,

    Extrinsic(Box<dyn ExtrinsicError>),
}

/// Error happens in extrinsic
pub trait ExtrinsicError {}

/// Extrinsic validation result
pub type Result<T> = std::result::Result<T, Error>;
