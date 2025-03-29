//! Report types

use crate::{
    service::{
        work::{ReportedWorkPackage, ReportedWorkPackageJson},
        RefineContext, RefineContextJson, RefineLoad, RefineLoadJson,
    },
    CoreIndex, ErasureRoot, ExportsRoot, Gas, OpaqueHash, ServiceId, WorkPackageHash,
};
use codec::Compact;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the result of a work execution.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum WorkExecResult {
    Ok(Vec<u8>),
    OutOfGas,
    Panic,
    BadCode,
    CodeOversize,
}

/// Represents the result of a work item.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct WorkResult {
    /// The service id
    pub service_id: ServiceId,

    /// The code hash
    #[json(hex)]
    pub code_hash: OpaqueHash,

    /// The payload hash
    #[json(hex)]
    pub payload_hash: OpaqueHash,

    /// The accumulate gas
    pub accumulate_gas: Gas,

    /// The result of the work item
    #[json(nested)]
    pub result: WorkExecResult,

    /// The refine load
    #[json(nested)]
    pub refine_load: RefineLoad,
}

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

// TODO: support enum in Json macro
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WorkExecResultJson {
    pub ok: Option<String>,
    #[serde(default = "default_some_unit")]
    pub out_of_gas: Option<()>,
    #[serde(default = "default_some_unit")]
    pub panic: Option<()>,
    #[serde(default = "default_some_unit")]
    pub bad_code: Option<()>,
    #[serde(default = "default_some_unit")]
    pub code_oversize: Option<()>,
}

fn default_some_unit() -> Option<()> {
    Some(())
}

impl Json<WorkExecResultJson> for WorkExecResult {
    fn to_json(self) -> WorkExecResultJson {
        match self {
            WorkExecResult::Ok(v) => WorkExecResultJson {
                ok: Some(hex::encode(v)),
                ..Default::default()
            },
            WorkExecResult::OutOfGas => WorkExecResultJson {
                out_of_gas: Some(()),
                ..Default::default()
            },
            WorkExecResult::Panic => WorkExecResultJson {
                panic: Some(()),
                ..Default::default()
            },
            WorkExecResult::BadCode => WorkExecResultJson {
                bad_code: Some(()),
                ..Default::default()
            },
            WorkExecResult::CodeOversize => WorkExecResultJson {
                code_oversize: Some(()),
                ..Default::default()
            },
        }
    }

    fn from_json(json: WorkExecResultJson) -> anyhow::Result<Self> {
        if let Some(ok) = json.ok {
            Ok(WorkExecResult::Ok(hex::decode(
                ok.trim_start_matches("0x"),
            )?))
        } else if json.out_of_gas.is_none() {
            Ok(WorkExecResult::OutOfGas)
        } else if json.panic.is_none() {
            Ok(WorkExecResult::Panic)
        } else if json.bad_code.is_none() {
            Ok(WorkExecResult::BadCode)
        } else {
            Ok(WorkExecResult::CodeOversize)
        }
    }
}
