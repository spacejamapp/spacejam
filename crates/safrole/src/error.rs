use serde::{Deserialize, Serialize};

/// Represents the CustomErrorCode enumeration.
#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    /// Timeslot value must be strictly monotonic
    BadSlot,
    UnexpectedTicket,
    BadTicketOrder,
    BadTicketProof,
    BadTicketAttempt,
    Reserved,
    DuplicateTicket,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
