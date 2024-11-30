//! Ticket types

use crate::misc::*;

/// Represents a unique identifier for a ticket.
pub type TicketId = OpaqueHash; // Corresponds to OpaqueHash

/// Represents an attempt to use a ticket.
pub type TicketAttempt = u8; // Corresponds to U8

/// Represents a ticket envelope containing an attempt and a signature.
pub struct TicketEnvelope {
    pub attempt: TicketAttempt,
    pub signature: BandersnatchRingVrfSignature, // Corresponds to BandersnatchRingVrfSignature
}

/// Represents the body of a ticket, containing an ID and an attempt.
pub struct TicketBody {
    pub id: TicketId,
    pub attempt: TicketAttempt,
}

/// Represents an accumulator of tickets.
pub struct TicketsAccumulator {
    pub tickets: Vec<TicketBody>, // SIZE(0..epoch-length) OF TicketBody
}

/// Represents either tickets or keys.
pub enum TicketsOrKeys {
    Tickets(Vec<TicketBody>),      // SIZE(epoch-length) OF TicketBody
    Keys(Vec<BandersnatchPublic>), // SIZE(epoch-length) OF BandersnatchPublic
}

/// Represents the extrinsic data for tickets.
pub struct TicketsExtrinsic {
    pub tickets: Vec<TicketEnvelope>, // SIZE(0..16) OF TicketEnvelope
}
