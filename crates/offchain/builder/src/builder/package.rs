//! Work package builder traits and implementations

use anyhow::Result;
use crate::builder::item::DefaultWorkItemBuilder;
use score::service::{RefineContext, WorkItem, WorkPackage};

/// Trait for building work packages
pub trait Builder: Send + Sync {
    /// Create a new work package with the given context and authorization
    fn new_package(
        &self,
        auth_token: Vec<u8>,
        auth_code_host: score::ServiceId,
        auth_code_hash: score::OpaqueHash,
        auth_config: Vec<u8>,
        context: RefineContext,
    ) -> Result<WorkPackageBuilder>;
    
    /// Finalize and build the work package
    fn build(&self, builder: WorkPackageBuilder) -> Result<WorkPackage>;
    
    /// Validate a work package according to Gray Paper constraints
    fn validate(&self, package: &WorkPackage) -> Result<()>;
}

/// Builder for constructing work packages incrementally
pub struct WorkPackageBuilder {
    /// Authorization token
    pub auth_token: Vec<u8>,
    /// Host service ID for authorization code
    pub auth_code_host: score::ServiceId,
    /// Authorization code hash
    pub auth_code_hash: score::OpaqueHash,
    /// Authorization configuration
    pub auth_config: Vec<u8>,
    /// Refine context
    pub context: RefineContext,
    /// Work items
    pub items: Vec<WorkItem>,
}

impl WorkPackageBuilder {
    /// Add a work item to the package
    pub fn add_item(&mut self, item: WorkItem) -> Result<&mut Self> {
        self.items.push(item);
        Ok(self)
    }
    
    /// Create a new work item builder
    pub fn new_item(
        &self,
        service: score::ServiceId,
        code_hash: score::OpaqueHash,
        payload: Vec<u8>,
    ) -> DefaultWorkItemBuilder {
        DefaultWorkItemBuilder::new(service, code_hash, payload)
    }
}