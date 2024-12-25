//! maybe generate this trait from build script

use crate::extrinsic::{
    AssurancesExtrinsic, DisputesExtrinsic, GuaranteesExtrinsic, PreimagesExtrinsic,
    TicketsExtrinsic,
};
use crate::validator::{Context, Result};
use std::sync::Arc;

/// Validate extrinsic
pub trait ValidateExtrinsic {
    /// Validate assurances extrinsic
    fn validate_assurances(context: Context, assurances: Arc<AssurancesExtrinsic>) -> Result<()>;
    /// Validate disputes extrinsic
    fn validate_disputes(context: Context, disputes: Arc<DisputesExtrinsic>) -> Result<()>;
    /// Validate preimages extrinsic
    fn validate_preimages(context: Context, preimages: Arc<PreimagesExtrinsic>) -> Result<()>;
    /// Validate guarantees extrinsic
    fn validate_guarantees(context: Context, guarantees: Arc<GuaranteesExtrinsic>) -> Result<()>;
    /// Validate tickets extrinsic
    fn validate_tickets(context: Context, tickets: Arc<TicketsExtrinsic>) -> Result<()>;
}
