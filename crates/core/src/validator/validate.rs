//! maybe generate this trait from build script
#![allow(async_fn_in_trait)]

use crate::extrinsic::{
    AssurancesExtrinsic, DisputesExtrinsic, GuaranteesExtrinsic, PreimagesExtrinsic,
    TicketsExtrinsic,
};
use crate::validator::{Context, Result};
use std::sync::Arc;

/// Extrinsic validator
pub trait ValidateExtrinsic:
    ValidateAssurance + ValidateDispute + ValidatePreimage + ValidateGuarantee + ValidateTicket
{
}

/// Validate assurances
#[trait_variant::make(Send)]
pub trait ValidateAssurance {
    async fn validate_assurances(
        context: Context,
        assurances: Arc<AssurancesExtrinsic>,
    ) -> Result<()>;
}

/// Validate disputes
#[trait_variant::make(Send)]
pub trait ValidateDispute {
    async fn validate_disputes(context: Context, disputes: Arc<DisputesExtrinsic>) -> Result<()>;
}

/// Validate preimages
#[trait_variant::make(Send)]
pub trait ValidatePreimage {
    async fn validate_preimages(context: Context, preimages: Arc<PreimagesExtrinsic>)
        -> Result<()>;
}

/// Validate guarantees
#[trait_variant::make(Send)]
pub trait ValidateGuarantee {
    async fn validate_guarantees(
        context: Context,
        guarantees: Arc<GuaranteesExtrinsic>,
    ) -> Result<()>;
}

/// Validate tickets
#[trait_variant::make(Send)]
pub trait ValidateTicket {
    async fn validate_tickets(context: Context, tickets: Arc<TicketsExtrinsic>) -> Result<()>;
}
