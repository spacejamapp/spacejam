//! Report types

use crate::misc::*;
use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents the result of a work execution.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct WorkExecResult {
    #[json(hex)]
    pub ok: Option<Vec<u8>>,
    pub out_of_gas: Option<()>,
    pub panic: Option<()>,
    pub bad_code: Option<()>,
    pub code_oversize: Option<()>,
}

/// Represents the result of a work item.
#[derive(Debug, Serialize, Deserialize, Json)]
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
#[derive(Debug, Serialize, Deserialize, Json)]
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
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct SegmentRootLookupItem {
    #[json(hex)]
    pub work_package_hash: WorkPackageHash,
    #[json(hex)]
    pub segment_tree_root: OpaqueHash,
}

/// Represents a work report.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct WorkReport {
    #[json(nested)]
    pub package_spec: WorkPackageSpec,
    #[json(nested)]
    pub context: RefineContext,
    pub core_index: CoreIndex,
    #[json(hex)]
    pub authorizer_hash: OpaqueHash,
    #[json(hex)]
    pub auth_output: Vec<u8>,
    #[json(nested)]
    pub segment_root_lookup: Vec<SegmentRootLookupItem>,
    #[json(nested)]
    pub results: Vec<WorkResult>,
}
