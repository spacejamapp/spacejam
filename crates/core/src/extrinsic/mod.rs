use crate::OpaqueHash;
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {assurance::*, dispute::*, guarantee::*, preimage::*, ticket::*};

mod assurance;
pub mod dispute;
mod guarantee;
mod preimage;
pub mod ticket;

/// Represents an extrinsic in the system.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Default, Clone)]
pub struct Extrinsic {
    /// The tickets
    ///
    /// Note that the maximum number of tickets is `K = 16`.
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
