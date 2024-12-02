//! Ticket types

use crate::misc::*;
use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents a unique identifier for a ticket.
pub type TicketId = OpaqueHash; // Corresponds to OpaqueHash

/// Represents an attempt to use a ticket.
pub type TicketAttempt = u8; // Corresponds to U8

/// Represents a ticket envelope containing an attempt and a signature.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct TicketEnvelope {
    pub attempt: TicketAttempt,
    #[json(hex)]
    #[serde(with = "codec")]
    pub signature: BandersnatchRingVrfSignature,
}

/// Represents the body of a ticket, containing an ID and an attempt.
#[derive(Debug, Serialize, Deserialize, Json, Copy, Clone, Default, PartialEq, Eq)]
pub struct TicketBody {
    #[json(hex)]
    pub id: TicketId,
    pub attempt: TicketAttempt,
}

/// Represents an accumulator of tickets.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct TicketsAccumulator {
    #[json(nested)]
    pub tickets: Vec<TicketBody>,
}

/// Represents either tickets or keys.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TicketsOrKeys {
    Tickets(Vec<TicketBody>),
    Keys(Vec<BandersnatchPublic>),
}

/// Represents the extrinsic data for tickets.
pub type TicketsExtrinsic = Vec<TicketEnvelope>;
