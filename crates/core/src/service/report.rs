//! Report types

use crate::{
    service::{
        RefineContext, RefineContextJson, WorkPackageSpec, WorkPackageSpecJson, WorkResult,
        WorkResultJson,
    },
    CoreIndex, OpaqueHash,
};
use codec::Compact;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a work report.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct WorkReport {
    /// The package spec
    #[json(nested)]
    pub package_spec: WorkPackageSpec,

    /// The context
    #[json(nested)]
    pub context: RefineContext,

    /// The core index
    pub core_index: CoreIndex,

    /// The authorizer hash
    #[json(hex)]
    pub authorizer_hash: OpaqueHash,

    /// The auth output
    #[json(hex)]
    pub auth_output: Vec<u8>,

    /// The reported work packages
    #[json(nested)]
    #[serde(alias = "segment_root_lookup")]
    pub reported: Vec<ReportedWorkPackage>,

    /// The results of the work items
    #[json(nested)]
    pub results: Vec<WorkResult>,

    /// The auth gas used
    #[json(compact)]
    pub auth_gas_used: Compact<u64>,
}

/// Represents a reported work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct ReportedWorkPackage {
    /// The hash
    #[json(hex)]
    #[serde(alias = "work_package_hash")]
    pub hash: OpaqueHash,

    /// The exports root
    #[json(hex)]
    #[serde(alias = "segment_tree_root")]
    pub exports_root: OpaqueHash,
}
