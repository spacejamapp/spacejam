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

/// Default builder for constructing work items
#[derive(Debug, Clone)]
pub struct DefaultWorkItemBuilder {
    service: score::ServiceId,
    code_hash: score::OpaqueHash,
    payload: Vec<u8>,
    refine_gas_limit: score::Gas,
    accumulate_gas_limit: score::Gas,
    import_segments: Vec<ImportSpec>,
    extrinsic: Vec<ExtrinsicSpec>,
    export_count: u16,
}

impl DefaultWorkItemBuilder {
    /// Create a new work item builder
    pub fn new(
        service: score::ServiceId,
        code_hash: score::OpaqueHash,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            service,
            code_hash,
            payload,
            refine_gas_limit: 0,
            accumulate_gas_limit: 0,
            import_segments: Vec::new(),
            extrinsic: Vec::new(),
            export_count: 0,
        }
    }
}

impl ItemBuilder for DefaultWorkItemBuilder {
    fn refine_gas_limit(mut self, gas: score::Gas) -> Self {
        self.refine_gas_limit = gas;
        self
    }
    
    fn accumulate_gas_limit(mut self, gas: score::Gas) -> Self {
        self.accumulate_gas_limit = gas;
        self
    }
    
    fn add_import(mut self, tree_root: score::OpaqueHash, index: u16) -> Result<Self> {
        self.import_segments.push(ImportSpec { tree_root, index });
        Ok(self)
    }
    
    fn add_extrinsic(mut self, hash: score::OpaqueHash, len: u32) -> Result<Self> {
        self.extrinsic.push(ExtrinsicSpec { hash, len });
        Ok(self)
    }
    
    fn export_count(mut self, count: u16) -> Self {
        self.export_count = count;
        self
    }
    
    fn build(self) -> WorkItem {
        // TODO: This is a temporary workaround until ImportSpec and ExtrinsicSpec
        // are publicly exposed from spacejam-core. Currently we can't construct
        // WorkItem with these private types.
        // 
        // The proper solution would be to either:
        // 1. Make work module public in spacejam-core, or
        // 2. Re-export ImportSpec and ExtrinsicSpec from service module
        unimplemented!("WorkItem construction blocked by private types in spacejam-core")
    }
}