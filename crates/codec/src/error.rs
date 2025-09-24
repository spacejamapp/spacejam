//! general errors

use crate::String;
use core::fmt::Display;

/// Error type for serde-jam
#[derive(Debug)]
pub enum Error {
    /// Any error from `anyhow`
    Anyhow(anyhow::Error),
    /// Invalid length
    InvalidLength {
        /// Expected length
        expected: usize,
        /// Got length
        got: usize,
    },
    /// Invalid input
    InvalidInput(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Anyhow(e) => e.fmt(f),
            Self::InvalidLength { expected, got } => {
                write!(f, "Invalid length: expected {expected}, got {got}")
            }
            Self::InvalidInput(s) => write!(f, "Invalid input: {s}"),
        }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Anyhow(e) => e.source(),
            _ => None,
        }
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Anyhow(anyhow::anyhow!("{msg}"))
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Anyhow(anyhow::anyhow!("{msg}"))
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Anyhow(err)
    }
}

/// Result type for serde-jam
pub type Result<T> = core::result::Result<T, Error>;
