//! Work item builder traits and implementations

use anyhow::Result;
use score::service::WorkItem;

/// Import specification for a work item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSpec {
    /// The tree root
    pub tree_root: score::OpaqueHash,
    /// The index
    pub index: u16,
}

/// Extrinsic specification for a work item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtrinsicSpec {
    /// The hash
    pub hash: score::OpaqueHash,
    /// The length
    pub len: u32,
}

/// Trait for building work items
pub trait ItemBuilder {
    /// Set refine gas limit
    fn refine_gas_limit(self, gas: score::Gas) -> Self;

    /// Set accumulate gas limit
    fn accumulate_gas_limit(self, gas: score::Gas) -> Self;

    /// Add an import segment
    fn add_import(self, tree_root: score::OpaqueHash, index: u16) -> Result<Self>
    where
        Self: Sized;

    /// Add an extrinsic
    fn add_extrinsic(self, hash: score::OpaqueHash, len: u32) -> Result<Self>
    where
        Self: Sized;

    /// Set export count
    fn export_count(self, count: u16) -> Self;

    /// Build the work item
    fn build(self) -> WorkItem;
}
