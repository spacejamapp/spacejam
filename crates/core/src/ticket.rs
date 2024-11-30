//! Ticket types

use crate::misc::*;
use core_derive::Json;
use scale::{Decode, Encode};

/// Represents a unique identifier for a ticket.
pub type TicketId = OpaqueHash; // Corresponds to OpaqueHash

/// Represents an attempt to use a ticket.
pub type TicketAttempt = u8; // Corresponds to U8

/// Represents a ticket envelope containing an attempt and a signature.
#[derive(Debug, Encode, Decode, Json)]
pub struct TicketEnvelope {
    pub attempt: TicketAttempt,
    #[json(hex)]
    pub signature: BandersnatchRingVrfSignature,
}

/// Represents the body of a ticket, containing an ID and an attempt.
#[derive(Debug, Encode, Decode, Json)]
pub struct TicketBody {
    pub id: TicketId,
    pub attempt: TicketAttempt,
}

/// Represents an accumulator of tickets.
#[derive(Debug, Encode, Decode, Json)]
pub struct TicketsAccumulator {
    #[json(nested)]
    pub tickets: Vec<TicketBody>,
}

/// Represents either tickets or keys.
#[derive(Debug, Encode, Decode)]
pub enum TicketsOrKeys {
    Tickets(Vec<TicketBody>),
    Keys(Vec<BandersnatchPublic>),
}

/// Represents the extrinsic data for tickets.
#[derive(Debug, Encode, Decode, Json)]
pub struct TicketsExtrinsic {
    #[json(nested)]
    pub tickets: Vec<TicketEnvelope>,
}
