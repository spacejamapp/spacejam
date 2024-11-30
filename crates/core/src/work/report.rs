//! Report types

use crate::misc::*;
use core_derive::Json;
use scale::{Decode, Encode};

/// Represents the result of a work execution.
#[derive(Debug, Encode, Decode, Json)]
pub struct WorkExecResult {
    // TODO: support Option in JSON derive
    pub ok: Option<Vec<u8>>,
    pub out_of_gas: Option<()>,
    pub panic: Option<()>,
    pub bad_code: Option<()>,
    pub code_oversize: Option<()>,
}

/// Represents the result of a work item.
#[derive(Debug, Encode, Decode, Json)]
pub struct WorkResult {
    pub service_id: ServiceId,
    #[json(hex)]
    pub code_hash: OpaqueHash,
    #[json(hex)]
    pub payload_hash: OpaqueHash,
    pub gas: Gas,
    #[json(nested)]
    pub result: WorkExecResult,
}

/// Represents the specification of a work package.
#[derive(Debug, Encode, Decode, Json)]
pub struct WorkPackageSpec {
    #[json(hex)]
    pub hash: WorkPackageHash,
    pub length: u32,
    #[json(hex)]
    pub erasure_root: ErasureRoot,
    #[json(hex)]
    pub exports_root: ExportsRoot,
    pub exports_count: u16,
}

/// Represents an item in the segment root lookup.
#[derive(Debug, Encode, Decode, Json)]
pub struct SegmentRootLookupItem {
    #[json(hex)]
    pub work_package_hash: WorkPackageHash,
    #[json(hex)]
    pub segment_tree_root: OpaqueHash,
}

/// Represents a work report.
pub struct WorkReport {
    pub package_spec: WorkPackageSpec,
    pub context: RefineContext,
    pub core_index: CoreIndex,
    pub authorizer_hash: OpaqueHash,
    pub auth_output: Vec<u8>,
    pub segment_root_lookup: Vec<SegmentRootLookupItem>,
    pub results: Vec<WorkResult>,
}
