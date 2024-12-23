use crate::EPOCH_LENGTH;
use crate::VALIDATORS_COUNT;
use crate::{
    extrinsic::*, BandersnatchPublic, BandersnatchVrfSignature, Ed25519Public, Entropy, HeaderHash,
    OpaqueHash, StateRoot, TimeSlot, ValidatorIndex,
};
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
    /// The parent block hash (H_p)
    #[json(hex)]
    pub parent: HeaderHash,
    /// The parent state root (H_r)
    #[json(hex)]
    pub parent_state_root: StateRoot,
    /// The extrinsic hash (H_x)
    #[json(hex)]
    pub extrinsic_hash: OpaqueHash,
    /// The slot of the block
    pub slot: TimeSlot,
    /// The epoch mark (H_e)
    #[json(nested)]
    pub epoch_mark: Option<EpochMark>,
    /// The winning tickets marker (H_w)
    #[json(Option<Vec<TicketBodyJson>>)]
    pub tickets_mark: Option<TicketsMark>,
    /// The offenders mark (H_o)
    #[json(hex)]
    pub offenders_mark: Vec<Ed25519Public>,
    /// The author index (H_i)
    pub author_index: ValidatorIndex,
    /// The entropy source (H_v)
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub entropy_source: BandersnatchVrfSignature,
    /// The seal (H_s)
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub seal: BandersnatchVrfSignature,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            parent: HeaderHash::default(),
            parent_state_root: StateRoot::default(),
            extrinsic_hash: OpaqueHash::default(),
            slot: TimeSlot::default(),
            epoch_mark: None,
            tickets_mark: None,
            offenders_mark: vec![],
            author_index: ValidatorIndex::default(),
            entropy_source: [0; 96],
            seal: [0; 96],
        }
    }
}
