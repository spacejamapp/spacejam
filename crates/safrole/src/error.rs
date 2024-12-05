use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents the CustomErrorCode enumeration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    /// Timeslot value must be strictly monotonic
    BadSlot,
    /// Unexpected ticket
    UnexpectedTicket,
    /// Invalid ticket order
    BadTicketOrder,
    /// Invalid ticket ring proof
    BadTicketProof,
    /// Invalid ticket attempt value
    BadTicketAttempt,
    /// Reserved
    Reserved,
    /// Duplicate ticket
    DuplicateTicket,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl Json<Error> for Error {
    fn to_json(self) -> Error {
        self
    }

    fn from_json(json: Error) -> anyhow::Result<Self> {
        Ok(json)
    }
}

/// Result type for the safrole crate
pub type Result<T> = core::result::Result<T, Error>;
