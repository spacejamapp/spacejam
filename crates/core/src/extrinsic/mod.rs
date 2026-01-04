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
    /// Returns the hash of the extrinsic
    #[cfg(feature = "blake2")]
    pub fn hash(&self) -> crate::OpaqueHash {
        let guarantees_data: Vec<([u8; 32], [u8; 4], &Vec<ValidatorSignature>)> = self
            .guarantees
            .iter()
            .map(|guarantee| {
                let work_report_hash = crate::blake2b(&codec::encode(&guarantee.report));
                let slot_bytes = guarantee.slot.to_le_bytes();
                (work_report_hash, slot_bytes, &guarantee.signatures)
            })
            .collect();
        let g: Vec<u8> = codec::encode(&guarantees_data);
        let hashes = &[
            codec::encode(&self.tickets),
            codec::encode(&self.preimages),
            g,
            codec::encode(&self.assurances),
            codec::encode(&self.disputes),
        ]
        .map(|component| crate::blake2b(&component))
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        crate::blake2b(hashes.as_slice())
    }
}
