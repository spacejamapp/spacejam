use crate::{
    service::{RefineContext, RefineContextJson},
    Gas, OpaqueHash, ServiceId,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a work package in the system.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct WorkPackage {
    #[json(hex)]
    pub authorization: Vec<u8>,
    pub auth_code_host: ServiceId,
    #[json(nested)]
    pub authorizer: Authorizer,
    #[json(nested)]
    pub context: RefineContext,
    #[json(nested)]
    pub items: Vec<WorkItem>,
}

/// Represents a reported work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct ReportedWorkPackage {
    #[json(hex)]
    #[serde(alias = "work_package_hash")]
    pub hash: OpaqueHash,
    #[json(hex)]
    #[serde(alias = "segment_tree_root")]
    pub exports_root: OpaqueHash,
}

/// Represents an individual work item within a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct WorkItem {
    pub service: ServiceId,
    #[json(hex)]
    pub code_hash: OpaqueHash,
    #[json(hex)]
    pub payload: Vec<u8>,
    pub refine_gas_limit: Gas,
    pub accumulate_gas_limit: Gas,
    #[json(nested)]
    pub import_segments: Vec<ImportSpec>,
    #[json(nested)]
    pub extrinsic: Vec<ExtrinsicSpec>,
    pub export_count: u16,
}

/// Represents an import specification for a work item.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct ImportSpec {
    #[json(hex)]
    pub tree_root: OpaqueHash,
    pub index: u16,
}

/// Represents an extrinsic specification for a work item.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct ExtrinsicSpec {
    #[json(hex)]
    pub hash: OpaqueHash,
    pub len: u32,
}

/// Represents an authorizer for a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct Authorizer {
    #[json(hex)]
    pub code_hash: OpaqueHash,
    #[json(hex)]
    pub params: Vec<u8>,
}
