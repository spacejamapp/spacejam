use crate::{
    service::{RefineContext, RefineContextJson},
    ErasureRoot, ExportsRoot, Gas, OpaqueHash, ServiceId, WorkPackageHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the specification of a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct WorkPackageSpec {
    /// The hash
    #[json(hex)]
    pub hash: WorkPackageHash,

    /// The length
    pub length: u32,

    /// The erasure root
    #[json(hex)]
    pub erasure_root: ErasureRoot,

    /// The exports root
    #[json(hex)]
    pub exports_root: ExportsRoot,

    /// The exports count
    pub exports_count: u16,
}

/// Represents a work package in the system.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct WorkPackage {
    /// The authorization
    #[json(hex)]
    pub authorization: Vec<u8>,

    /// The auth code host
    pub auth_code_host: ServiceId,

    /// The authorizer
    #[json(nested)]
    pub authorizer: Authorizer,

    /// The context
    #[json(nested)]
    pub context: RefineContext,

    /// The items
    #[json(nested)]
    pub items: Vec<WorkItem>,
}

/// Represents an individual work item within a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct WorkItem {
    /// The service
    pub service: ServiceId,

    /// The code hash
    #[json(hex)]
    pub code_hash: OpaqueHash,

    /// The payload
    #[json(hex)]
    pub payload: Vec<u8>,

    /// The refine gas limit
    pub refine_gas_limit: Gas,

    /// The accumulate gas limit
    pub accumulate_gas_limit: Gas,

    /// The import segments
    #[json(nested)]
    pub import_segments: Vec<ImportSpec>,

    /// The extrinsic
    #[json(nested)]
    pub extrinsic: Vec<ExtrinsicSpec>,

    /// The export count
    pub export_count: u16,
}

/// Represents an import specification for a work item.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct ImportSpec {
    /// The tree root
    #[json(hex)]
    pub tree_root: OpaqueHash,

    /// The index
    pub index: u16,
}

/// Represents an extrinsic specification for a work item.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct ExtrinsicSpec {
    /// The hash
    #[json(hex)]
    pub hash: OpaqueHash,

    /// The length
    pub len: u32,
}

/// Represents an authorizer for a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct Authorizer {
    /// The code hash
    #[json(hex)]
    pub code_hash: OpaqueHash,

    /// The params
    #[json(hex)]
    pub params: Vec<u8>,
}
