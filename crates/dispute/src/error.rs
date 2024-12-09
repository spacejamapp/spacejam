//! Error codes for disputes

use codec::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    AlreadyJudged = 0,
    BadVoteSplit = 1,
    VerdictsNotSortedUnique = 2,
    JudgementsNotSortedUnique = 3,
    CulpritsNotSortedUnique = 4,
    FaultsNotSortedUnique = 5,
    NotEnoughCulprits = 6,
    NotEnoughFaults = 7,
    CulpritsVerdictNotBad = 8,
    FaultVerdictWrong = 9,
    OffenderAlreadyReported = 10,
    BadJudgementAge = 11,
    BadValidatorIndex = 12,
    BadSignature = 13,
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
