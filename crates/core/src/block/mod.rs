use crate::block::header::{Header, HeaderJson};
use crate::extrinsic::*;
use crate::misc::HeaderHash;
pub use history::BlocksHistory;
use serde::{Deserialize, Serialize};
use spacejson::Json;

pub mod header;
pub mod history;

#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Default)]
pub struct Extrinsic {
    /// The tickets
    #[json(Vec<TicketEnvelopeJson>)]
    pub tickets: TicketsExtrinsic,
    /// The preimages
    #[json(Vec<PreimageJson>)]
    pub preimages: PreimagesExtrinsic,
    /// The guarantees
    #[json(Vec<ReportGuaranteeJson>)]
    pub guarantees: GuaranteesExtrinsic,
    /// The assurances
    #[json(Vec<AvailAssuranceJson>)]
    pub assurances: AssurancesExtrinsic,
    /// The disputes
    #[json(nested)]
    pub disputes: DisputesExtrinsic,
}

/// Represents a block in the system.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Block {
    /// The header of the block
    #[json(nested)]
    pub header: Header,
    /// The extrinsic of the block
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
