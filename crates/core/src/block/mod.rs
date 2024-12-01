use crate::block::header::{Header, HeaderJson};
use crate::dispute::*;
use crate::misc::*;
use crate::ticket::*;
use json::Json;
use scale::{Decode, Encode};

pub mod header;
pub mod history;

#[derive(Debug, Encode, Decode, Json)]
pub struct Extrinsic {
    #[json(Vec<TicketEnvelopeJson>)]
    pub tickets: TicketsExtrinsic,
    #[json(Vec<PreimageJson>)]
    pub preimages: PreimagesExtrinsic,
    #[json(Vec<ReportGuaranteeJson>)]
    pub guarantees: GuaranteesExtrinsic,
    #[json(Vec<AvailAssuranceJson>)]
    pub assurances: AssurancesExtrinsic,
    #[json(nested)]
    pub disputes: DisputesExtrinsic,
}

/// Represents a block in the system.
#[derive(Debug, Encode, Decode, Json)]
pub struct Block {
    #[json(nested)]
    pub header: Header,
    #[json(nested)]
    pub extrinsic: Extrinsic,
}
