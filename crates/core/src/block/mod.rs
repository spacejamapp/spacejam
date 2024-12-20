use crate::block::header::{Header, HeaderJson};
use crate::extrinsic::*;
use crate::misc::HeaderHash;
pub use history::BlocksHistory;
use serde::{Deserialize, Serialize};
use spacejson::Json;

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

impl Block {
    /// Returns the hash of the block
    pub fn hash(&self) -> anyhow::Result<HeaderHash> {
        let encoded = codec::encode(&self.header)?;
        Ok(crypto::blake2b(&encoded))
    }
}
