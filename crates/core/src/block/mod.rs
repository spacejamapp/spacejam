use crate::block::header::Header;
use crate::dispute::*;
use crate::misc::*;
use crate::ticket::*;

pub mod header;
pub mod history;

pub struct Extrinsic {
    pub tickets: TicketsExtrinsic,
    pub preimages: PreimagesExtrinsic,
    pub guarantees: GuaranteesExtrinsic,
    pub assurances: AssurancesExtrinsic,
    pub disputes: DisputesExtrinsic,
}

/// Represents a block in the system.
pub struct Block {
    pub header: Header,       // Corresponds to Header
    pub extrinsic: Extrinsic, // Corresponds to Extrinsic
}
