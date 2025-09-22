//! Metadata for a PVM program.

use serde::{Deserialize, Serialize};

/// Conventional metadata for a PVM program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConventionalMetadata {
    /// Information on a crate, useful for building conventional medata of type 0.
    Info(CrateInfo),
}

impl ConventionalMetadata {
    /// Get the name of the crate.
    pub fn info(&self) -> &CrateInfo {
        match self {
            Self::Info(info) => info,
        }
    }
}

impl Default for ConventionalMetadata {
    fn default() -> Self {
        Self::Info(CrateInfo::default())
    }
}

/// Information on a crate, useful for building conventional medata of type 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CrateInfo {
    /// The name of the crate.
    pub name: String,
    /// The version of the crate.
    pub version: String,
    /// The license of the crate.
    pub license: String,
    /// The authors of the crate.
    pub authors: Vec<String>,
}
