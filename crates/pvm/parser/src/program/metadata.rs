//! Metadata for a PVM program.

use serde::{Deserialize, Serialize};

/// Conventional metadata for a PVM program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConventionalMetadata {
    /// Information on a crate, useful for building conventional medata of type 0.
    Info(CrateInfo),
}

/// Information on a crate, useful for building conventional medata of type 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
