//! Work result types

use crate::{
    service::{RefineLoad, RefineLoadJson},
    Gas, OpaqueHash, ServiceId,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

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

/// Represents the result of a work execution. (11.7)
///
/// TODO: need to fix the graypaper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkExecResult {
    Ok(Vec<u8>),
    /// ∞ denoting an out-of-gas error
    OutOfGas,
    /// ☇ denoting an unexpected program termination
    Panic,
    /// ⊥ denoting an invalid exports error
    ///
    /// FIXME: put this here is not correct, but it's a workaround for the old tests
    InvalidExports,
    /// ⊥ denoting an code oversize error
    CodeOversize,
    /// ⊥ denoting an invalid code error
    BadCode,
}

// TODO: support enum in Json macro
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkExecResultJson {
    pub ok: Option<String>,
    #[serde(default = "default_some_unit")]
    pub out_of_gas: Option<()>,
    #[serde(default = "default_some_unit")]
    pub panic: Option<()>,
    #[serde(default = "default_some_unit")]
    pub invalid_exports: Option<()>,
    #[serde(default = "default_some_unit")]
    pub code_oversize: Option<()>,
    #[serde(default = "default_some_unit")]
    pub bad_code: Option<()>,
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
            WorkExecResult::InvalidExports => WorkExecResultJson {
                invalid_exports: Some(()),
                ..Default::default()
            },
            WorkExecResult::CodeOversize => WorkExecResultJson {
                code_oversize: Some(()),
                ..Default::default()
            },
            WorkExecResult::BadCode => WorkExecResultJson {
                bad_code: Some(()),
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
