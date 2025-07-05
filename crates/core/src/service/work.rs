use crate::{
    service::{PackageValidation, RefineContext, RefineContextJson},
    ErasureRoot, ExportsRoot, Gas, OpaqueHash, ServiceId, WorkPackageHash,
};
use anyhow::Result;
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
    /// (j) The authorization token
    #[json(hex)]
    pub authorization: Vec<u8>,

    /// (h) The auth code host
    pub auth_code_host: ServiceId,

    /// (u, a) The authorizer
    #[json(nested)]
    pub authorizer: Authorizer,

    /// (c) The context
    #[json(nested)]
    pub context: RefineContext,

    /// (w) The items
    #[json(nested)]
    pub items: Vec<WorkItem>,
}

impl WorkPackage {
    /// Validate the work package according to Gray Paper specifications
    pub fn validate(&self) -> Result<PackageValidation> {
        let validation = PackageValidation::new(self);
        validation.validate()?;
        Ok(validation)
    }
}

/// Represents an individual work item within a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct WorkItem {
    /// (s) The service
    pub service: ServiceId,

    /// (h) The code hash
    #[json(hex)]
    pub code_hash: OpaqueHash,

    /// (y) The payload
    #[json(hex)]
    pub payload: Vec<u8>,

    /// (g) The refine gas limit
    pub refine_gas_limit: Gas,

    /// (a) The accumulate gas limit
    pub accumulate_gas_limit: Gas,

    /// (i) The import segments
    ///
    /// MAX=W_M=3072
    #[json(nested)]
    pub import_segments: Vec<ImportSpec>,

    /// (x) The extrinsic
    ///
    /// MAX=T=128
    #[json(nested)]
    pub extrinsic: Vec<ExtrinsicSpec>,

    /// (e) The export count
    ///
    /// MAX=W_X=3072
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

impl Authorizer {
    /// Compute the authorizer hash
    ///
    /// FIXME: shall we hash it after encoding?
    pub fn hash(&self) -> OpaqueHash {
        crypto::blake2b(&[self.code_hash.as_ref(), &self.params].concat())
    }
}
