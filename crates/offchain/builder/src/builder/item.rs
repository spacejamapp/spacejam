//! Work item builder traits and implementations

use anyhow::Result;
use score::service::{ExtrinsicSpec, WorkItem};

/// Trait for building work items
pub trait Builder: Sized {
    /// Set refine gas limit
    fn refine_gas_limit(self, gas: score::Gas) -> Self;

    /// Set accumulate gas limit
    fn accumulate_gas_limit(self, gas: score::Gas) -> Self;

    /// Add an import segment
    fn add_import(self, tree_root: score::OpaqueHash, index: u16) -> Result<Self>;

    /// Add an extrinsic from raw data
    ///
    /// Automatically computes hash and length per Gray Paper specification.
    fn add_extrinsic(self, extrinsic: ExtrinsicSpec) -> Result<Self>;

    /// Set export count
    fn export_count(self, count: u16) -> Self;

    /// Build the work item
    fn build(self) -> Result<WorkItem>;
}
