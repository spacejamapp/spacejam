use crate::{extrinsic::*, HeaderHash, OpaqueHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    header::{Header, HeaderJson},
    history::BlocksHistory,
};

pub mod header;
pub mod history;

#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Default, Clone)]
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

impl Extrinsic {
    /// Returns the hash of the extrinsic
    pub fn hash(&self) -> anyhow::Result<OpaqueHash> {
        let encoded = codec::encode(&self)?;
        Ok(crypto::blake2b(&encoded))
    }
}

/// Represents a block in the system.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Default, Clone)]
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
