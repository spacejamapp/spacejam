use crate::misc::*;
use crate::ticket::*;
use codec::Json;
use serde::{Deserialize, Serialize};

/// Represents the epoch mark in a block header.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct EpochMark {
    #[json(hex)]
    pub entropy: Entropy,
    #[json(hex)]
    pub tickets_entropy: Entropy,
    #[json(hex)]
    pub validators: Vec<BandersnatchPublic>,
}

/// Represents the tickets mark in a block header.
pub type TicketsMark = Vec<TicketBody>;

/// Represents the header of a block.
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct Header {
    #[json(hex)]
    pub parent: HeaderHash,
    #[json(hex)]
    pub parent_state_root: StateRoot,
    #[json(hex)]
    pub extrinsic_hash: OpaqueHash,
    pub slot: TimeSlot,
    #[json(nested)]
    pub epoch_mark: Option<EpochMark>,
    #[json(Option<Vec<TicketBodyJson>>)]
    pub tickets_mark: Option<TicketsMark>,
    #[json(hex)]
    pub offenders_mark: Vec<Ed25519Public>,
    pub author_index: ValidatorIndex,
    #[json(hex)]
    #[serde(with = "codec")]
    pub entropy_source: BandersnatchVrfSignature,
    #[json(hex)]
    #[serde(with = "codec")]
    pub seal: BandersnatchVrfSignature,
}
