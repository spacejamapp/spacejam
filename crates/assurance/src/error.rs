//! Error types for the assurance module.

use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Error codes for the assurance module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    BadAttestationParent,
    BadValidatorIndex,
    CoreNotEngaged,
    BadSignature,
    NotSortedOrUniqueAssurers,
}

impl Json<Error> for Error {
    fn to_json(self) -> Self {
        self.clone()
    }

    fn from_json(json: Self) -> anyhow::Result<Self> {
        Ok(json)
    }
}

/// Result type for the assurance module.
pub type Result<T> = std::result::Result<T, Error>;
