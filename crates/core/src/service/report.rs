//! Report types

use crate::{
    CoreIndex, OpaqueHash, ServiceId, WorkPackageHash,
    service::{
        RefineContext, RefineContextJson, WorkPackageSpec, WorkPackageSpecJson, WorkResult,
        WorkResultJson,
    },
};
use anyhow;
use serde::{Deserialize, Serialize};
use service::vm::Operand;
use spacejson::Json;
use std::collections::BTreeMap;

/// Represents a work report.
#[derive(Debug, Serialize, Deserialize, PartialEq, Json, Eq, Clone, Default)]
pub struct WorkReport {
    /// The package spec
    #[json(nested)]
    #[serde(alias = "package_spec")]
    pub spec: WorkPackageSpec,

    /// The refine context
    #[json(nested)]
    pub context: RefineContext,

    /// The core index
    #[serde(with = "codec::compact")]
    pub core_index: CoreIndex,

    /// The authorizer hash
    #[json(hex)]
    pub authorizer_hash: OpaqueHash,

    /// The auth gas used
    #[serde(with = "codec::compact")]
    pub auth_gas_used: u64,

    /// The auth output
    #[json(hex)]
    pub auth_output: Vec<u8>,

    /// The segment root lookup directory
    #[serde(alias = "segment_root_lookup")]
    #[json(array(key = "work_package_hash", value = "segment_tree_root"))]
    pub lookup: BTreeMap<WorkPackageHash, OpaqueHash>,

    /// The results of the work items
    #[json(nested)]
    pub results: Vec<WorkResult>,
}

impl WorkReport {
    /// Check if the work report is immediate
    pub fn is_immediate(&self) -> bool {
        self.lookup.is_empty() && self.context.prerequisites.is_empty()
    }

    /// Get the operands
    pub fn operands(&self, service: ServiceId) -> Vec<Operand> {
        let mut operands = vec![];
        for work in self.results.iter() {
            if work.service_id != service {
                continue;
            }

            operands.push(Operand {
                package: self.spec.hash,
                exports_root: self.spec.exports_root,
                authorizer_hash: self.authorizer_hash,
                auth_output: self.auth_output.clone(),
                payload: work.payload_hash,
                gas: work.accumulate_gas,
                data: work.result.clone(),
            });
        }
        operands
    }
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

/// The ready record
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ReadyReport {
    /// The report
    #[json(nested)]
    pub report: WorkReport,

    /// The dependencies
    #[json(Vec<String>)]
    pub dependencies: Vec<WorkPackageHash>,
}
