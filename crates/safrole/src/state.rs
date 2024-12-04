//! SAFRole types

use crate::Error;
use anyhow::Result;
use codec::Json;
use score::{
    block::header::{EpochMark, EpochMarkJson, TicketsMark},
    misc::{
        BandersnatchRingCommitment, EntropyBuffer, OpaqueHash, ValidatorDataJson, ValidatorsData,
    },
    ticket::{
        TicketBodyJson, TicketsAccumulator, TicketsExtrinsic, TicketsOrKeys, TicketsOrKeysJson,
    },
};
use serde::{Deserialize, Serialize};

/// Represents the State structure.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Json, Clone)]
pub struct State {
    /// Most recent block's timeslot.
    pub tau: u32,
    /// Entropy accumulator and epochal randomness.
    ///
    /// graypaper reference: 6.21
    #[json(Vec<String>)]
    pub eta: EntropyBuffer,
    /// Previous epoch's validators
    #[json(Vec<ValidatorDataJson>)]
    pub lambda: ValidatorsData,
    /// Current epoch's validators
    #[json(Vec<ValidatorDataJson>)]
    pub kappa: ValidatorsData,
    /// Validators to be drawn from next
    #[json(Vec<ValidatorDataJson>)]
    pub iota: ValidatorsData,
    /// Next epoch's validators
    #[json(Vec<ValidatorDataJson>)]
    pub gamma_k: ValidatorsData,
    /// Bandersnatch ring commitment
    #[serde(with = "codec")]
    #[json(hex)]
    pub gamma_z: BandersnatchRingCommitment,
    /// Sealing-key series of the current epoch
    #[json(nested)]
    pub gamma_s: TicketsOrKeys,
    /// Sealing-key contest ticket accumulator
    #[json(Vec<TicketBodyJson>)]
    pub gamma_a: TicketsAccumulator,
}

impl State {
    /// Enacts an epoch change and updates the entropy accumulator.
    pub fn enact(
        &mut self,
        slot: u32,
        entropy: OpaqueHash,
        _extrinsic: TicketsExtrinsic,
    ) -> Result<std::result::Result<Markers, Error>> {
        if slot <= self.tau {
            return Ok(Err(Error::BadSlot));
        }

        let new_epoch: bool = (slot / score::EPOCH_LENGTH) > (self.tau / score::EPOCH_LENGTH);

        if new_epoch {
            self.rotate_keys();
        }

        // Update the entropy accumulator
        self.update_eta(new_epoch, entropy);

        // Update the epoch
        self.tau = slot;

        Ok(Ok(Markers {
            epoch_mark: self.collect_epoch_marker(new_epoch),
            tickets_mark: self.collect_tickets_marker(new_epoch),
        }))
    }

    /// Updates the entropy accumulator.
    ///
    /// graypaper reference: 6.4
    pub fn update_eta(&mut self, new_epoch: bool, entropy: OpaqueHash) {
        // graypaper reference: 6.23
        //
        // eta'_e = H(eta_e || eta'_(e-1))
        if new_epoch {
            let historical_eta = self.eta.clone();
            self.eta[1..].copy_from_slice(&historical_eta[..3]);
        }

        // graypaper reference: 6.22
        //
        // eta'_0 = H(eta_0 || Y(H_v))
        let eta_0 = crypto::blake2b(&[self.eta[0], entropy].concat());
        self.eta[0] = eta_0;
    }

    /// Calculates the epoch markers.
    ///
    /// graypaper reference: 6.6
    pub fn collect_epoch_marker(&self, new_epoch: bool) -> Option<EpochMark> {
        if !new_epoch {
            return None;
        }

        let next_epoch_validators: Vec<_> = self
            .gamma_k
            .iter()
            .map(|validator| validator.bandersnatch)
            .collect();

        let mut validators = [[0; 32]; score::VALIDATORS_COUNT as usize];
        validators.copy_from_slice(&next_epoch_validators);

        Some(EpochMark {
            entropy: self.eta[1],
            validators,
            tickets_entropy: self.eta[2],
        })
    }

    /// Calculates the tickets marker.
    ///
    /// graypaper reference: 6.6
    pub fn collect_tickets_marker(&self, _new_epoch: bool) -> Option<TicketsMark> {
        // TODO: conditions for epoch change
        //
        // graypaper reference: 6.28
        None
    }

    /// Rotates the keys for a new epoch.
    ///
    /// graypaper reference: 6.3
    /// graypaper formula: 6.13
    pub fn rotate_keys(&mut self) {
        // update previous epoch validators
        self.lambda = self.kappa.clone();
        // update current epoch validators
        self.kappa = self.gamma_k.clone();
        // update next epoch validators
        self.gamma_k = self.iota.clone();

        // update bandersnatch ring commitment
        let keys = self
            .gamma_k
            .iter()
            .map(|validator| validator.bandersnatch)
            .collect::<Vec<_>>();
        self.gamma_z = crypto::ring::commitment(keys);

        // TODO: graypaper reference: 6.14
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            tau: 0,
            eta: Default::default(),
            lambda: Default::default(),
            kappa: Default::default(),
            iota: Default::default(),
            gamma_k: Default::default(),
            gamma_z: [0u8; 144],
            gamma_s: Default::default(),
            gamma_a: Default::default(),
        }
    }
}

/// Represents the Output marks
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Json)]
pub struct Markers {
    /// New epoch marker
    #[json(nested)]
    pub epoch_mark: Option<EpochMark>,
    /// New tickets marker
    #[json(Option<Vec<TicketBodyJson>>)]
    pub tickets_mark: Option<TicketsMark>,
}
