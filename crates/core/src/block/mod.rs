use crate::block::header::{Header, HeaderJson};
use crate::dispute::*;
use crate::misc::*;
use crate::ticket::*;
use codec::Json;
use serde::{Deserialize, Serialize};

pub mod header;
pub mod history;

#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
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
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Block {
    #[json(nested)]
    pub header: Header,
    #[json(nested)]
    pub extrinsic: Extrinsic,
}
