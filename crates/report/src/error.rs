//! Error types for the reporting module.

use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Error codes for the reporting module.
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

/// Result type for the reporting module.
pub type Result<T> = std::result::Result<T, Error>;
