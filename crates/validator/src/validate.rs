//! maybe generate this trait from build script
#![allow(async_fn_in_trait)]

use crate::{Context, Error, ExtrinsicInMem, ExtrinsicType, Result};
use score::{
    consensus::Safrole,
    extrinsic::{
        AssurancesExtrinsic, DisputesExtrinsic, GuaranteesExtrinsic, PreimagesExtrinsic,
        TicketsExtrinsic,
    },
};
use std::sync::Arc;

/// Extrinsic validator
pub trait ValidateExtrinsic:
    ValidateAssurance + ValidateDispute + ValidatePreimage + ValidateGuarantee + ValidateTicket
{
    const ASSIGNMENT: ExtrinsicType;

    type Error: Into<Error>;

    /// Validate extrinsic, as a result:
    ///
    /// - returns state changes (safrole)
    /// - returns storage changes (TODO)
    async fn validate_extrinsic(
        &self,
        context: Context,
        extrinsic: &mut ExtrinsicInMem,
    ) -> Result<()> {
        match Self::ASSIGNMENT {
            ExtrinsicType::Assurances => {
                let Some(assurances) = extrinsic.assurances.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_assurances(context, assurances).await?;
                extrinsic.assurances = None;
                Ok(())
            }
            ExtrinsicType::Disputes => {
                let Some(disputes) = extrinsic.disputes.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_disputes(context, disputes).await?;
                extrinsic.disputes = None;
                Ok(())
            }
            ExtrinsicType::Preimages => {
                let Some(preimages) = extrinsic.preimages.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_preimages(context, preimages).await?;
                extrinsic.preimages = None;
                Ok(())
            }
            ExtrinsicType::Guarantees => {
                let Some(guarantees) = extrinsic.guarantees.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_guarantees(context, guarantees).await?;
                extrinsic.guarantees = None;
                Ok(())
            }
            ExtrinsicType::Tickets => {
                let Some(tickets) = extrinsic.tickets.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_tickets(context, tickets).await?;
                extrinsic.tickets = None;
                Ok(())
            }
        }
    }
}

/// Validate assurances
pub trait ValidateAssurance {
    async fn validate_assurances(
        &self,
        context: Context,
        assurances: Arc<AssurancesExtrinsic>,
    ) -> Result<Safrole>;
}

/// Validate disputes
pub trait ValidateDispute {
    async fn validate_disputes(
        &self,
        context: Context,
        disputes: Arc<DisputesExtrinsic>,
    ) -> Result<Safrole>;
}

/// Validate preimages
pub trait ValidatePreimage {
    async fn validate_preimages(
        &self,
        context: Context,
        preimages: Arc<PreimagesExtrinsic>,
    ) -> Result<Safrole>;
}

/// Validate guarantees
pub trait ValidateGuarantee {
    async fn validate_guarantees(
        &self,
        context: Context,
        guarantees: Arc<GuaranteesExtrinsic>,
    ) -> Result<Safrole>;
}

/// Validate tickets
pub trait ValidateTicket {
    async fn validate_tickets(
        &self,
        context: Context,
        tickets: Arc<TicketsExtrinsic>,
    ) -> Result<Safrole>;
}
