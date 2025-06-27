//! Error codes for disputes

use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    AlreadyJudged,
    BadAuditorKey,
    BadGuarantorKey,
    BadVoteSplit,
    VerdictsNotSortedUnique,
    JudgementsNotSortedUnique,
    CulpritsNotSortedUnique,
    FaultsNotSortedUnique,
    NotEnoughCulprits,
    NotEnoughFaults,
    CulpritsVerdictNotBad,
    FaultVerdictWrong,
    OffenderAlreadyReported,
    BadJudgementAge,
    BadValidatorIndex,
    BadSignature,
    VerdictNotExists,
    NotEnoughValidators,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Json<Error> for Error {
    fn to_json(self) -> Error {
        self
    }

    fn from_json(json: Error) -> anyhow::Result<Self> {
        Ok(json)
    }
}

/// Result type for disputes
pub type Result<T> = core::result::Result<T, Error>;
