use crate::{
    block::header::{EpochMark, TicketsMark},
    extrinsic::{TicketBody, TicketsAccumulator, TicketsOrKeys},
    validator::ValidatorsData,
    BandersnatchRingCommitment, Ed25519Public, OpaqueHash,
};
use serde::{Deserialize, Serialize};

/// Safrole consensus state
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Safrole {
    /// Sealing-key contest ticket accumulator (gamma_a)
    pub accumulator: TicketsAccumulator,
    /// Next epoch's validators (gamma_k)
    pub validators: ValidatorsData,
    /// Sealing-key series of the current epoch (gamma_s)
    pub series: TicketsOrKeys,
    /// Bandersnatch ring commitment (gamma_z)
    #[serde(with = "codec::bytes")]
    pub ring_commitment: BandersnatchRingCommitment,
}

impl Safrole {
    /// (γ_k') Returns the validators for the next epoch.
    pub fn next(
        &self,
        new_epoch: bool,
        drawn: &ValidatorsData,
        offenders: &[Ed25519Public],
    ) -> ValidatorsData {
        if !new_epoch {
            return self.validators.clone();
        }

        drawn
            .iter()
            .map(|validator| {
                if offenders.contains(&validator.ed25519) {
                    Default::default()
                } else {
                    validator.clone()
                }
            })
            .collect()
    }

    /// (γ_z') Returns the bandersnatch ring commitment.
    pub fn commitment(&self, new_epoch: bool) -> BandersnatchRingCommitment {
        if !new_epoch {
            return self.ring_commitment;
        }

        let keys = self
            .validators
            .iter()
            .map(|validator| validator.bandersnatch)
            .collect::<Vec<_>>();
        crypto::ring::commitment(keys)
    }

    /// Collects the epoch mark.
    pub fn epoch_mark(&self, new_epoch: bool, entropy: &[OpaqueHash; 4]) -> Option<EpochMark> {
        if !new_epoch {
            return None;
        }

        let next_epoch_validators: Vec<_> = self
            .validators
            .iter()
            .map(|validator| validator.bandersnatch)
            .collect();

        let mut validators = [[0; 32]; crate::VALIDATORS_COUNT as usize];
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
            || prev_slot_phase >= crate::CONTEST_DURATION
            || curr_slot_phase < crate::CONTEST_DURATION
            || self.accumulator.len() != crate::EPOCH_LENGTH as usize
        {
            return None;
        }

        // Apply Z function to gamma_a (outside-in sequencing)
        let mut tickets = [TicketBody::default(); crate::EPOCH_LENGTH as usize];
        tickets.copy_from_slice(&self.tickets());
        Some(tickets)
    }

    /// Sequences the tickets
    pub fn tickets(&self) -> Vec<TicketBody> {
        let mut ordered_tickets = Vec::with_capacity(self.accumulator.len());
        let mid = self.accumulator.len() / 2;

        for i in 0..mid {
            ordered_tickets.push(self.accumulator[i]);
            if i + mid < self.accumulator.len() {
                ordered_tickets.push(self.accumulator[self.accumulator.len() - 1 - i]);
            }
        }

        ordered_tickets
    }
}

impl Default for Safrole {
    fn default() -> Self {
        Self {
            accumulator: vec![],
            validators: vec![],
            series: TicketsOrKeys::default(),
            ring_commitment: [0u8; 144],
        }
    }
}
