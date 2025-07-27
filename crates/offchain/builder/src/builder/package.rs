//! Work package builder traits and implementations

use anyhow::Result;
use score::service::{RefineContext, WorkItem, WorkPackage};

/// Trait for building work packages
pub trait Builder: Send + Sync {
    /// Create a new work package builder with the given context and authorization
    fn new_package(
        auth_token: Vec<u8>,
        auth_code_host: score::ServiceId,
        auth_code_hash: score::OpaqueHash,
        auth_config: Vec<u8>,
        context: RefineContext,
    ) -> Self;

    /// Add a work item to the package
    fn add_item(&mut self, item: WorkItem) -> Result<&mut Self>;

    /// Finalize and build the work package
    fn build(self) -> Result<WorkPackage>;

    /// Validate a work package according to Gray Paper constraints
    fn validate(package: &WorkPackage) -> Result<()>;
}
