//! Work result types

use crate::{
    Gas, OpaqueHash, ServiceId, String, Vec,
    service::{RefineLoad, RefineLoadJson},
};
pub use json::WorkExecResultJson;
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(C)]
pub enum WorkExecResult {
    Ok(Vec<u8>),
    /// ∞ denoting an out-of-gas error
    OutOfGas,
    /// ☇ denoting an unexpected program termination
    Panic,
    /// ⊥ the number of exports made was invalidly reported
    InvalidExports,
    /// the size of the digest (refinement output) would
    /// cross the acceptable limit
    InvalidDigest,
    /// (BAD) the third indicates that the service’s code
    /// was not available for lookup in state at the posterior state
    /// of the lookup-anchor block
    BadCode,
    /// (BIG) the code was available but was beyond the maximum size
    CodeOversize,
}

/// The result of is-authorized invocation (ΨI)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Executed {
    /// The output
    pub data: Vec<u8>,

    /// The reason
    pub exec: WorkExecResult,

    /// The gas used
    pub gas: Gas,
}

impl Executed {
    /// Create a new executed result
    pub fn new(data: Vec<u8>, exec: WorkExecResult, gas: Gas) -> Self {
        Self { data, exec, gas }
    }

    /// Check if the execution is successful
    pub fn is_ok(&self) -> bool {
        matches!(self.exec, WorkExecResult::Ok(_))
    }
}

/// The result of refine invocation (ΨR)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Refined {
    /// The executed result
    pub executed: Executed,

    /// The imports
    pub segments: Vec<Segment>,
}

impl Refined {
    /// Create a new refined result
    pub fn new(executed: Executed, segments: Vec<[u8; crate::SEGMENT_SIZE]>) -> Self {
        Self {
            executed,
            segments: segments.iter().map(|s| Segment(*s)).collect(),
        }
    }
}

/// A segment of the import segments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Segment(#[serde(with = "codec::bytes")] pub [u8; crate::SEGMENT_SIZE]);

mod json {
    use super::WorkExecResult;
    use crate::String;
    use serde::{Deserialize, Serialize};
    use spacejson::Json;

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
        pub invalid_digest: Option<()>,
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
                WorkExecResult::InvalidExports => WorkExecResultJson {
                    invalid_exports: Some(()),
                    ..Default::default()
                },
                WorkExecResult::InvalidDigest => WorkExecResultJson {
                    invalid_digest: Some(()),
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
                Ok(WorkExecResult::Ok(
                    hex::decode(ok.trim_start_matches("0x"))
                        .map_err(|e| anyhow::anyhow!("{e:?}"))?,
                ))
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
}
