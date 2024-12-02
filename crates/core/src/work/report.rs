//! Report types

use crate::misc::*;
use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents the result of a work execution.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkExecResult {
    Ok(Vec<u8>),
    OutOfGas,
    Panic,
    BadCode,
    CodeOversize,
}

/// Represents the result of a work item.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct WorkResult {
    pub service_id: ServiceId,
    #[json(hex)]
    pub code_hash: OpaqueHash,
    #[json(hex)]
    pub payload_hash: OpaqueHash,
    pub gas: Gas,
    #[json(nested)]
    pub result: WorkExecResult,
}

/// Represents the specification of a work package.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct WorkPackageSpec {
    #[json(hex)]
    pub hash: WorkPackageHash,
    pub length: u32,
    #[json(hex)]
    pub erasure_root: ErasureRoot,
    #[json(hex)]
    pub exports_root: ExportsRoot,
    pub exports_count: u16,
}

/// Represents an item in the segment root lookup.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct SegmentRootLookupItem {
    #[json(hex)]
    pub work_package_hash: WorkPackageHash,
    #[json(hex)]
    pub segment_tree_root: OpaqueHash,
}

/// Represents a work report.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct WorkReport {
    #[json(nested)]
    pub package_spec: WorkPackageSpec,
    #[json(nested)]
    pub context: RefineContext,
    pub core_index: CoreIndex,
    #[json(hex)]
    pub authorizer_hash: OpaqueHash,
    #[json(hex)]
    pub auth_output: Vec<u8>,
    #[json(nested)]
    pub segment_root_lookup: Vec<SegmentRootLookupItem>,
    #[json(nested)]
    pub results: Vec<WorkResult>,
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
