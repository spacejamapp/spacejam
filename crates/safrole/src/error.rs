use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents the CustomErrorCode enumeration.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub struct ErrorJson {
    bad_slot: Option<()>,
    unexpected_ticket: Option<()>,
    bad_ticket_order: Option<()>,
    bad_ticket_proof: Option<()>,
    bad_ticket_attempt: Option<()>,
    reserved: Option<()>,
    duplicate_ticket: Option<()>,
}

impl Json<ErrorJson> for Error {
    fn to_json(self) -> ErrorJson {
        match self {
            Error::BadSlot => ErrorJson {
                bad_slot: Some(()),
                ..Default::default()
            },
            Error::UnexpectedTicket => ErrorJson {
                unexpected_ticket: Some(()),
                ..Default::default()
            },
            Error::BadTicketOrder => ErrorJson {
                bad_ticket_order: Some(()),
                ..Default::default()
            },
            Error::BadTicketProof => ErrorJson {
                bad_ticket_proof: Some(()),
                ..Default::default()
            },
            Error::BadTicketAttempt => ErrorJson {
                bad_ticket_attempt: Some(()),
                ..Default::default()
            },
            Error::Reserved => ErrorJson {
                reserved: Some(()),
                ..Default::default()
            },
            Error::DuplicateTicket => ErrorJson {
                duplicate_ticket: Some(()),
                ..Default::default()
            },
        }
    }

    fn from_json(json: ErrorJson) -> Result<Self, anyhow::Error> {
        if json.bad_slot.is_some() {
            return Ok(Error::BadSlot);
        }
        if json.unexpected_ticket.is_some() {
            return Ok(Error::UnexpectedTicket);
        }
        if json.bad_ticket_order.is_some() {
            return Ok(Error::BadTicketOrder);
        }
        if json.bad_ticket_proof.is_some() {
            return Ok(Error::BadTicketProof);
        }
        if json.bad_ticket_attempt.is_some() {
            return Ok(Error::BadTicketAttempt);
        }
        if json.reserved.is_some() {
            return Ok(Error::Reserved);
        }
        if json.duplicate_ticket.is_some() {
            return Ok(Error::DuplicateTicket);
        }
        Err(anyhow::anyhow!("Unknown error type"))
    }
}
