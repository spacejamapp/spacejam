//! Safrole types

use crate::{
    BandersnatchRingCommitment, Ed25519Public, OpaqueHash,
    block::header::{EValidator, EpochMark, TicketsMark},
    extrinsic::{TicketBody, TicketBodyJson, TicketsAccumulator, TicketsOrKeys, TicketsOrKeysJson},
};
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use validator::{
    ValidatorData, ValidatorDataJson, ValidatorIter, Validators, ValidatorsData, ValidatorsJson,
};

mod validator;

/// Safrole consensus state
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Json)]
pub struct Safrole {
    /// Next epoch's validators (gamma_k)
    #[json(Vec<ValidatorDataJson>)]
    pub validators: ValidatorsData,

    /// Bandersnatch ring commitment (gamma_z)
    #[serde(with = "codec::bytes")]
    #[json(hex)]
    pub ring_commitment: BandersnatchRingCommitment,

    /// Sealing-key series of the current epoch (gamma_s)
    #[json(nested)]
    pub series: TicketsOrKeys,

    /// Sealing-key contest ticket accumulator (gamma_a)
    #[json(Vec<TicketBodyJson>)]
    pub accumulator: TicketsAccumulator,
}

impl Safrole {
    /// (γ_k') Returns the validators for the next epoch.
    pub fn next(&self, drawn: &ValidatorsData, offenders: &[Ed25519Public]) -> ValidatorsData {
        // Apply blacklist filter Φ(ι)
        let mut next = [ValidatorData::default(); crate::VALIDATORS_COUNT as usize];
        for (i, validator) in drawn.iter().enumerate() {
            next[i] = if offenders.contains(&validator.ed25519) {
                ValidatorData::default()
            } else {
                *validator
            };
        }

        next
    }

    /// Collects the epoch mark.
    pub fn epoch_mark(&self, entropy: &[OpaqueHash; 4]) -> Option<EpochMark> {
        let next_epoch_validators: Vec<_> = self
            .validators
            .iter()
            .map(|validator| EValidator {
                bandersnatch: validator.bandersnatch,
                ed25519: validator.ed25519,
            })
            .collect();

        let mut validators = [EValidator::default(); crate::VALIDATORS_COUNT as usize];
        validators.copy_from_slice(&next_epoch_validators);

        Some(EpochMark {
            entropy: entropy[1],
            validators,
            tickets_entropy: entropy[2],
        })
    }

    /// Collects the tickets mark.
    pub fn tickets_mark(&self, tau: u32, slot: u32) -> Option<TicketsMark> {
        let curr_epoch = slot / crate::EPOCH_LENGTH;
        let prev_epoch = tau / crate::EPOCH_LENGTH;
        let curr_slot_phase = slot % crate::EPOCH_LENGTH;
        let prev_slot_phase = tau % crate::EPOCH_LENGTH;

        // Return None if:
        // 1. Different epochs (e' ≠ e)
        // 2. Previous slot not before submission period (m ≥ Y)
        // 3. Current slot not after submission period (m' < Y)
        // 4. Accumulator not full (|gamma_a| ≠ E)
        if curr_epoch != prev_epoch
            || prev_slot_phase >= crate::TICKET_SUBMISSION_PERIOD
            || curr_slot_phase < crate::TICKET_SUBMISSION_PERIOD
            || self.accumulator.len() != crate::EPOCH_LENGTH as usize
        {
            return None;
        }

        // Apply Z function to gamma_a (outside-in sequencing)
        let mut tickets = [TicketBody::default(); crate::EPOCH_LENGTH as usize];
        tickets.copy_from_slice(&TicketBody::sequence(&self.accumulator));
        Some(tickets)
    }
}

impl Default for Safrole {
    fn default() -> Self {
        Self {
            accumulator: vec![],
            validators: Default::default(),
            series: TicketsOrKeys::default(),
            ring_commitment: [0u8; 144],
        }
    }
}
