use crate::extrinsic::*;
use crate::misc::*;
use crate::EPOCH_LENGTH;
use crate::VALIDATORS_COUNT;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the epoch mark in a block header.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct EpochMark {
    #[json(hex)]
    pub entropy: Entropy,
    #[json(hex)]
    pub tickets_entropy: Entropy,
    #[json(Vec<String>)]
    pub validators: [BandersnatchPublic; VALIDATORS_COUNT as usize],
}

/// Represents the tickets mark in a block header.
pub type TicketsMark = [TicketBody; EPOCH_LENGTH as usize];

/// Represents the header of a block.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
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
    #[serde(with = "codec::bytes")]
    pub entropy_source: BandersnatchVrfSignature,
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub seal: BandersnatchVrfSignature,
}
