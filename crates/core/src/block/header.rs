use crate::misc::*;
use crate::ticket::*;

/// Represents the epoch mark in a block header.
pub struct EpochMark {
    pub entropy: Entropy,                    // Corresponds to Entropy
    pub tickets_entropy: Entropy,            // Corresponds to Entropy
    pub validators: Vec<BandersnatchPublic>, // Sequence of BandersnatchPublic
}

/// Represents the tickets mark in a block header.
pub struct TicketsMark {
    pub tickets: Vec<TicketBody>, // Corresponds to TicketBody
}

/// Represents the header of a block.
pub struct Header {
    pub parent: HeaderHash,                       // Corresponds to HeaderHash
    pub parent_state_root: StateRoot,             // Corresponds to StateRoot
    pub extrinsic_hash: OpaqueHash,               // Corresponds to OpaqueHash
    pub slot: TimeSlot,                           // Corresponds to TimeSlot
    pub epoch_mark: Option<EpochMark>,            // Corresponds to EpochMark (OPTIONAL)
    pub tickets_mark: Option<TicketsMark>,        // Corresponds to TicketsMark (OPTIONAL)
    pub offenders_mark: Vec<Ed25519Public>,       // Corresponds to Ed25519Public
    pub author_index: ValidatorIndex,             // Corresponds to ValidatorIndex
    pub entropy_source: BandersnatchVrfSignature, // Corresponds to BandersnatchVrfSignature
    pub seal: BandersnatchVrfSignature,           // Corresponds to BandersnatchVrfSignature
}
