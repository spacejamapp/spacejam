use crate::misc::*;

pub mod report;

/// Represents a work package in the system.
pub struct WorkPackage {
    pub authorization: Vec<u8>, // Corresponds to ByteSequence
    pub auth_code_host: ServiceId,
    pub authorizer: ServiceInfo,
    pub context: RefineContext,
    pub items: Vec<WorkItem>, // List of work items
}

/// Represents an individual work item within a work package.
pub struct WorkItem {
    pub service: ServiceId,
    pub code_hash: OpaqueHash,
    pub payload: Vec<u8>, // Corresponds to ByteSequence
    pub gas_limit: Gas,
    pub import_segments: Vec<ImportSpec>, // List of import specifications
    pub extrinsic: Vec<ExtrinsicSpec>,    // List of extrinsic specifications
    pub export_count: u16,
}

/// Represents an import specification for a work item.
pub struct ImportSpec {
    pub tree_root: OpaqueHash,
    pub index: u16,
}

/// Represents an extrinsic specification for a work item.
pub struct ExtrinsicSpec {
    pub hash: OpaqueHash,
    pub len: u32,
}
