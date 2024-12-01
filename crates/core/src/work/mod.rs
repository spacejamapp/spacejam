use crate::misc::*;
use codec::Json;
use serde::{Deserialize, Serialize};

pub mod report;

/// Represents a work package in the system.
#[derive(Debug, Serialize, Deserialize, Json)]
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

/// Represents an individual work item within a work package.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct WorkItem {
    pub service: ServiceId,
    #[json(hex)]
    pub code_hash: OpaqueHash,
    #[json(hex)]
    pub payload: Vec<u8>,
    pub gas_limit: Gas,
    #[json(nested)]
    pub import_segments: Vec<ImportSpec>,
    #[json(nested)]
    pub extrinsic: Vec<ExtrinsicSpec>,
    pub export_count: u16,
}

/// Represents an import specification for a work item.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct ImportSpec {
    #[json(hex)]
    pub tree_root: OpaqueHash,
    pub index: u16,
}

/// Represents an extrinsic specification for a work item.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct ExtrinsicSpec {
    #[json(hex)]
    pub hash: OpaqueHash,
    pub len: u32,
}

/// Represents an authorizer for a work package.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Authorizer {
    #[json(hex)]
    pub code_hash: OpaqueHash,
    #[json(hex)]
    pub params: Vec<u8>,
}
