//! Extrinsic types

use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {assurance::*, dispute::*, guarantee::*, preimage::*, ticket::*};

mod assurance;
pub mod dispute;
mod guarantee;
mod preimage;
pub mod ticket;

/// Represents extrinsics in a block.
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
    #[cfg(feature = "blake2")]
    /// Returns the hash of the extrinsic
    pub fn hash(&self) -> anyhow::Result<crate::OpaqueHash> {
        let encoded = codec::encode(&self)?;
        Ok(crypto::blake2b(&encoded))
    }
}
