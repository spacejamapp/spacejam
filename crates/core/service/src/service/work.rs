//! Work package related stuffs

use crate::{
    ErasureRoot, ExportsRoot, Gas, OpaqueHash, ServiceId, Vec, WorkPackageHash,
    service::RefineContext,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use {crate::String, crate::service::RefineContextJson, spacejson::Json};

/// Represents the specification of a work package.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct WorkPackageSpec {
    /// (p) The hash
    #[cfg_attr(feature = "json", json(hex))]
    pub hash: WorkPackageHash,

    /// (l) The length of the erasure bundle
    pub length: u32,

    /// (u) The erasure root
    #[cfg_attr(feature = "json", json(hex))]
    pub erasure_root: ErasureRoot,

    /// (e) The exports root (segment root)
    #[cfg_attr(feature = "json", json(hex))]
    pub exports_root: ExportsRoot,

    /// (n) The exports count
    pub exports_count: u16,
}

/// Represents a work package in the system.
///
/// TODO: embed token and host to the authorizer?
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct WorkPackage {
    /// (h) The auth code host
    pub auth_code_host: ServiceId,

    /// (u) The auth code hash
    #[cfg_attr(feature = "json", json(hex))]
    pub auth_code_hash: OpaqueHash,

    /// (c) The context
    #[cfg_attr(feature = "json", json(nested))]
    pub context: RefineContext,

    /// (j) The authorization token
    #[cfg_attr(feature = "json", json(hex))]
    pub authorization: Vec<u8>,

    /// (a) The authorizer
    #[cfg_attr(feature = "json", json(hex))]
    #[serde(alias = "authorizer_config")]
    pub config: Vec<u8>,

    /// (w) The items
    #[cfg_attr(feature = "json", json(nested))]
    pub items: Vec<WorkItem>,
}

/// Represents an individual work item within a work package.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct WorkItem {
    /// (s) The service
    pub service: ServiceId,

    /// (h) The code hash
    #[cfg_attr(feature = "json", json(hex))]
    pub code_hash: OpaqueHash,

    /// (g) The refine gas limit
    pub refine_gas_limit: Gas,

    /// (a) The accumulate gas limit
    pub accumulate_gas_limit: Gas,

    /// (e) The export count
    ///
    /// MAX=W_X=3072
    pub export_count: u16,

    /// (y) The payload
    #[cfg_attr(feature = "json", json(hex))]
    pub payload: Vec<u8>,

    /// (i) The import segments
    ///
    /// MAX=W_M=3072
    #[cfg_attr(feature = "json", json(nested))]
    pub import_segments: Vec<ImportSpec>,

    /// (x) The extrinsic
    ///
    /// MAX=T=128
    #[cfg_attr(feature = "json", json(nested))]
    pub extrinsic: Vec<ExtrinsicSpec>,
}

/// Represents an import specification for a work item.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct ImportSpec {
    /// The tree root
    #[cfg_attr(feature = "json", json(hex))]
    pub tree_root: OpaqueHash,

    /// The index
    pub index: u16,
}

/// Represents an extrinsic specification for a work item.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct ExtrinsicSpec {
    /// The hash
    #[cfg_attr(feature = "json", json(hex))]
    pub hash: OpaqueHash,

    /// The length
    pub len: u32,
}

#[cfg(feature = "blake2")]
impl WorkPackage {
    /// Compute the authorizer hash
    ///
    /// FIXME: shall we hash it after encoding?
    pub fn authorizer_hash(&self) -> OpaqueHash {
        crate::blake2b(&[self.auth_code_hash.as_ref(), &self.config].concat())
    }
}
