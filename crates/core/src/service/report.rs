//! Report types

use crate::{
    CoreIndex, OpaqueHash, ServiceId, WorkPackageHash,
    service::{
        RefineContext, RefineContextJson, WorkDigest, WorkDigestJson, WorkPackageSpec,
        WorkPackageSpecJson,
    },
};
use anyhow;
use serde::{Deserialize, Serialize};
use service::vm::Operand;
use spacejson::Json;
use std::collections::BTreeMap;

/// (11.2) Represents a work report.
#[derive(Debug, Serialize, Deserialize, PartialEq, Json, Eq, Clone, Default)]
pub struct WorkReport {
    /// (s) The package spec
    #[json(nested)]
    #[serde(alias = "package_spec")]
    pub spec: WorkPackageSpec,

    /// (c) The refine context
    #[json(nested)]
    pub context: RefineContext,

    /// (_c_) The core index
    #[serde(with = "codec::compact")]
    pub core_index: CoreIndex,

    /// (a) The authorizer hash
    #[json(hex)]
    pub authorizer_hash: OpaqueHash,

    /// (t) The auth output
    #[json(hex)]
    pub auth_output: Vec<u8>,

    /// (l) The segment root lookup directory
    #[serde(alias = "segment_root_lookup")]
    #[json(array(key = "work_package_hash", value = "segment_tree_root"))]
    pub lookup: BTreeMap<WorkPackageHash, OpaqueHash>,

    /// (d) The results of the work items
    #[json(nested)]
    pub results: Vec<WorkDigest>,

    /// (g) The auth gas used
    #[serde(with = "codec::compact")]
    pub auth_gas_used: u64,
}

impl WorkReport {
    /// Check if the work report is immediate
    pub fn is_immediate(&self) -> bool {
        self.lookup.is_empty() && self.context.prerequisites.is_empty()
    }

    /// (12.23) Get the operands
    pub fn operands(&self, service: ServiceId) -> Vec<Operand> {
        let mut operands = vec![];
        for work in self.results.iter() {
            if work.service_id != service {
                continue;
            }

            operands.push(Operand {
                // (l) The work execution result
                data: work.result.clone(),
                // (g) The accumulate gas
                gas: work.accumulate_gas,
                // (y) The payload hash
                package: self.spec.hash,
                // (t) The auth output
                auth_output: self.auth_output.clone(),
                // (e) The exports root
                exports_root: self.spec.exports_root,
                // (p) The package hash
                payload: work.payload_hash,
                // (a) The authorizer hash
                authorizer_hash: self.authorizer_hash,
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
