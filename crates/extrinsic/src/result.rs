//! Extrinsic Result

/// Extrinsic validation error
pub enum Error {
    /// Extrinsic is validated
    ExtrinsicValidated,
}

/// Extrinsic validation result
pub type Result<T> = std::result::Result<T, Error>;
