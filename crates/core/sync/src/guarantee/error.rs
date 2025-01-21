//! Error types for the reporting module.

use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Error codes for the reporting module.
///
/// NOTE:
///
/// Missing cases:
/// - import self
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    BadCoreIndex,
    FutureReportSlot,
    ReportEpochBeforeLast,
    InsufficientGuarantees,
    OutOfOrderGuarantee,
    NotSortedOrUniqueGuarantors,
    WrongAssignment,
    CoreEngaged,
    AnchorNotRecent,
    BadServiceId,
    BadCodeHash,
    DependencyMissing,
    DuplicatePackage,
    BadStateRoot,
    BadBeefyMmrRoot,
    CoreUnauthorized,
    BadValidatorIndex,
    WorkReportGasTooHigh,
    ServiceItemGasTooLow,
    TooManyDependencies,
    SegmentRootLookupInvalid,
    WorkReportTooBig,
    BadSignature,
}

impl Json<Error> for Error {
    fn to_json(self) -> Self {
        self.clone()
    }

    fn from_json(json: Self) -> anyhow::Result<Self> {
        Ok(json)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

/// Result type for the reporting module.
pub type Result<T> = std::result::Result<T, Error>;
