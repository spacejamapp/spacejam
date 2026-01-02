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
    #[cfg(all(feature = "blake2", feature = "merkle"))]
    pub fn hash(&self) -> crate::OpaqueHash {
        // Encode guarantees specially: g = encode([(hash(w), encode_4(t), a)])
        let g: Vec<u8> = codec::encode(
            &self
                .guarantees
                .iter()
                .map(|guarantee| {
                    let work_report_hash = crate::blake2b(&codec::encode(&guarantee.report));
                    let slot_bytes = (guarantee.slot as u32).to_le_bytes();
                    (work_report_hash, slot_bytes, &guarantee.signatures)
                })
                .collect::<Vec<_>>(),
        );

        // Build sequence a = [encode_T, encode_P, g, encode_A, encode_D]
        let a: Vec<Vec<u8>> = vec![
            codec::encode(&self.tickets),
            codec::encode(&self.preimages),
            g,
            codec::encode(&self.assurances),
            codec::encode(&self.disputes),
        ];

        // hash#(a) - binary merkle root
        let merkle_root = crypto::merkle::broot(a);
        crate::blake2b(&codec::encode(&merkle_root))
    }
}
