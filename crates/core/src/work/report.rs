//! Report types

use crate::misc::*;

/// Represents the result of a work execution.
pub struct WorkExecResult {
    pub ok: Option<Vec<u8>>,       // Corresponds to ByteSequence
    pub out_of_gas: Option<()>,    // Corresponds to NULL
    pub panic: Option<()>,         // Corresponds to NULL
    pub bad_code: Option<()>,      // Corresponds to NULL
    pub code_oversize: Option<()>, // Corresponds to NULL
}

/// Represents the result of a work item.
pub struct WorkResult {
    pub service_id: ServiceId,
    pub code_hash: OpaqueHash,
    pub payload_hash: OpaqueHash,
    pub gas: Gas,
    pub result: WorkExecResult,
}

/// Represents the specification of a work package.
pub struct WorkPackageSpec {
    pub hash: WorkPackageHash,
    pub length: u32,
    pub erasure_root: ErasureRoot,
    pub exports_root: ExportsRoot,
    pub exports_count: u16,
}

/// Represents an item in the segment root lookup.
pub struct SegmentRootLookupItem {
    pub work_package_hash: WorkPackageHash,
    pub segment_tree_root: OpaqueHash,
}

/// Represents a work report.
pub struct WorkReport {
    pub package_spec: WorkPackageSpec,
    pub context: RefineContext,
    pub core_index: CoreIndex,
    pub authorizer_hash: OpaqueHash,
    pub auth_output: Vec<u8>, // Corresponds to ByteSequence
    pub segment_root_lookup: Vec<SegmentRootLookupItem>, // Assuming SegmentRootLookupItem is defined elsewhere
    pub results: Vec<WorkResult>,                        // SIZE(1..4) OF WorkResult
}
