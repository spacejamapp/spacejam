//! Report types

use crate::{
    service::{
        RefineContext, RefineContextJson, WorkPackageSpec, WorkPackageSpecJson, WorkResult,
        WorkResultJson,
    },
    vm::Operand,
    CoreIndex, OpaqueHash, ServiceId, WorkPackageHash,
};
use codec::Compact;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a work report.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct WorkReport {
    /// The package spec
    #[json(nested)]
    #[serde(alias = "package_spec")]
    pub spec: WorkPackageSpec,

    /// The context
    #[json(nested)]
    pub context: RefineContext,

    /// The core index
    #[json(compact)]
    pub core_index: Compact<CoreIndex>,

    /// The authorizer hash
    #[json(hex)]
    pub authorizer_hash: OpaqueHash,

    /// The auth output
    #[json(hex)]
    #[serde(with = "codec::vlen")]
    pub auth_output: Vec<u8>,

    /// The segment root lookup directory
    #[json(nested)]
    #[serde(alias = "segment_root_lookup")]
    pub lookup: Vec<ReportedWorkPackage>,

    /// The results of the work items
    #[json(nested)]
    pub results: Vec<WorkResult>,

    /// The auth gas used
    #[json(compact)]
    pub auth_gas_used: Compact<u64>,
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
                data: work.result.clone(),
                erasure_root: self.spec.erasure_root,
                authorizer_output: self.auth_output.clone(),
                payload: work.payload_hash,
                hash: self.spec.hash,
                gas: work.accumulate_gas,
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
