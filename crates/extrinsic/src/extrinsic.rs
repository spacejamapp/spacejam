use score::extrinsic::{
    AssurancesExtrinsic, DisputesExtrinsic, GuaranteesExtrinsic, PreimagesExtrinsic,
    TicketsExtrinsic,
};
use std::sync::Arc;

/// Extrinsic in pool
///
/// storing extrinsic with smart pointers for avoiding memory allocation.
pub struct ExtrinsicInPool {
    /// Assurance extrinsic
    pub assurances: Arc<AssurancesExtrinsic>,
    /// Dispute extrinsic
    pub disputes: Arc<DisputesExtrinsic>,
    /// Preimage extrinsic
    pub preimages: Arc<PreimagesExtrinsic>,
    /// Guarantee extrinsic
    pub guarantees: Arc<GuaranteesExtrinsic>,
    /// Ticket extrinsic
    pub tickets: Arc<TicketsExtrinsic>,
}

/// Extrinsic in memory
///
/// If an extrinsic is validated, it will be set to `None`.
#[derive(Clone)]
pub struct ExtrinsicInMem {
    /// Assurance extrinsic
    pub assurances: Option<Arc<AssurancesExtrinsic>>,
    /// Dispute extrinsic
    pub disputes: Option<Arc<DisputesExtrinsic>>,
    /// Preimage extrinsic
    pub preimages: Option<Arc<PreimagesExtrinsic>>,
    /// Guarantee extrinsic
    pub guarantees: Option<Arc<GuaranteesExtrinsic>>,
    /// Ticket extrinsic
    pub tickets: Option<Arc<TicketsExtrinsic>>,
}

impl ExtrinsicInMem {
    /// Check if all extrinsic is validated
    pub fn is_validated(&self) -> bool {
        self.assurances.is_none()
            && self.disputes.is_none()
            && self.preimages.is_none()
            && self.guarantees.is_none()
            && self.tickets.is_none()
    }
}
