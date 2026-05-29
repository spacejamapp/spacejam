//! The specjam library
#![doc = include_str!("../README.md")]

pub use registry::{Entry, Registry};
pub use section::{Section, Trace};

mod registry;
mod section;

/// A general test vector
///
/// This is the main struct that represents a test vector
#[derive(Debug, Clone)]
pub struct Test {
    /// The scale of the test vectors
    pub scale: Option<Scale>,
    /// The section of the test vectors
    pub section: Section,
    /// The name of the test vector
    pub name: String,
    /// The input of the test vectors
    pub input: Payload,
    /// The output of the test vectors
    pub output: Payload,
}

impl Test {
    /// Whether the test vector is full
    pub fn is_full(&self) -> bool {
        self.scale == Some(Scale::Full)
    }
}

/// A test-vector payload.
#[derive(Debug, Clone)]
pub enum Payload {
    Json(String),
    Bin(Vec<u8>),
}

impl Payload {
    /// Borrow as JSON text, or `None` if binary.
    pub fn as_json(&self) -> Option<&str> {
        match self {
            Payload::Json(s) => Some(s),
            Payload::Bin(_) => None,
        }
    }

    /// Borrow as raw bytes, or `None` if JSON.
    pub fn as_bin(&self) -> Option<&[u8]> {
        match self {
            Payload::Bin(b) => Some(b),
            Payload::Json(_) => None,
        }
    }

    /// Borrow as JSON text; error if binary.
    pub fn expect_json(&self) -> anyhow::Result<&str> {
        self.as_json()
            .ok_or_else(|| anyhow::anyhow!("expected JSON payload, got binary"))
    }

    /// Borrow as raw bytes; error if JSON.
    pub fn expect_bin(&self) -> anyhow::Result<&[u8]> {
        self.as_bin()
            .ok_or_else(|| anyhow::anyhow!("expected binary payload, got JSON"))
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self::Json(String::new())
    }
}

/// The scale of the test vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// The test vectors are small
    Tiny,
    /// The test vectors are full
    Full,
}

impl AsRef<str> for Scale {
    fn as_ref(&self) -> &str {
        match self {
            Scale::Tiny => "tiny",
            Scale::Full => "full",
        }
    }
}
