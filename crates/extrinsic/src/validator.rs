//! Extension for validating extrinsic
#![allow(async_fn_in_trait)]

use crate::{extrinsic::ExtrinsicInMem, Error, ExtrinsicType, Result};
use score::{
    consensus::Safrole,
    extrinsic::{
        AssurancesExtrinsic, DisputesExtrinsic, GuaranteesExtrinsic, PreimagesExtrinsic,
        TicketsExtrinsic,
    },
};
use std::sync::Arc;

/// Extrinsic validator
pub trait Validator {
    const ASSIGNMENT: ExtrinsicType;

    type Error: Into<Error>;

    /// Validate extrinsic, as a result:
    ///
    /// - returns state changes (safrole)
    /// - returns storage changes (TODO)
    async fn validate(&self, safrole: Arc<Safrole>, extrinsic: &mut ExtrinsicInMem) -> Result<()> {
        match Self::ASSIGNMENT {
            ExtrinsicType::Assurances => {
                let Some(assurances) = extrinsic.assurances.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_assurances(safrole, assurances).await?;
                extrinsic.assurances = None;
                Ok(())
            }
            ExtrinsicType::Disputes => {
                let Some(disputes) = extrinsic.disputes.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_disputes(safrole, disputes).await?;
                extrinsic.disputes = None;
                Ok(())
            }
            ExtrinsicType::Preimages => {
                let Some(preimages) = extrinsic.preimages.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_preimages(safrole, preimages).await?;
                extrinsic.preimages = None;
                Ok(())
            }
            ExtrinsicType::Guarantees => {
                let Some(guarantees) = extrinsic.guarantees.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_guarantees(safrole, guarantees).await?;
                extrinsic.guarantees = None;
                Ok(())
            }
            ExtrinsicType::Tickets => {
                let Some(tickets) = extrinsic.tickets.clone() else {
                    return Err(Error::ExtrinsicValidated);
                };

                let _ = self.validate_tickets(safrole, tickets).await?;
                extrinsic.tickets = None;
                Ok(())
            }
        }
    }

    async fn validate_assurances(
        &self,
        safrole: Arc<Safrole>,
        _assurances: Arc<AssurancesExtrinsic>,
    ) -> Result<Safrole> {
        Ok((*safrole).clone())
    }

    async fn validate_disputes(
        &self,
        safrole: Arc<Safrole>,
        _disputes: Arc<DisputesExtrinsic>,
    ) -> Result<Safrole> {
        Ok((*safrole).clone())
    }

    async fn validate_preimages(
        &self,
        safrole: Arc<Safrole>,
        _preimages: Arc<PreimagesExtrinsic>,
    ) -> Result<Safrole> {
        Ok((*safrole).clone())
    }

    async fn validate_guarantees(
        &self,
        safrole: Arc<Safrole>,
        _guarantees: Arc<GuaranteesExtrinsic>,
    ) -> Result<Safrole> {
        Ok((*safrole).clone())
    }

    async fn validate_tickets(
        &self,
        safrole: Arc<Safrole>,
        _tickets: Arc<TicketsExtrinsic>,
    ) -> Result<Safrole> {
        Ok((*safrole).clone())
    }
}
