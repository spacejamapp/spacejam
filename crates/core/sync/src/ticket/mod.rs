//! Spacejam's SAFRole prototype

use score::{
    block::header::{EpochMark, TicketsMark},
    extrinsic::ticket::{TicketBody, TicketsExtrinsic, TicketsOrKeys},
    Block, OpaqueHash,
};
pub use {
    error::{Error, Result},
    state::{State, StateJson},
};

pub mod error;
pub mod state;

/// Validates tickets and updates the state
pub fn validate(
    state: &mut score::State,
    block: &Block,
    entropy: OpaqueHash,
) -> Result<(Option<EpochMark>, Option<TicketsMark>)> {
    let mut pstate: State = state.clone().into();
    let result = pstate.enact(block, entropy);
    pstate.apply(state);
    result
}

impl State {
    /// Enacts an epoch change and updates the entropy accumulator.
    pub fn enact(
        &mut self,
        block: &Block,
        entropy: OpaqueHash,
    ) -> Result<(Option<EpochMark>, Option<TicketsMark>)> {
        let slot = block.header.slot;
        let tickets = &block.extrinsic.tickets;

        let prev_state = self.clone();
        if slot <= self.tau {
            return Err(Error::BadSlot);
        }

        if slot % score::CONTEST_DURATION == 0 && !tickets.is_empty() {
            return Err(Error::UnexpectedTicket);
        }

        let epoch = slot / score::EPOCH_LENGTH;
        let new_epoch: bool = epoch > (self.tau / score::EPOCH_LENGTH);

        if new_epoch {
            self.rotate_keys();
        }

        self.update_eta(new_epoch, entropy);
        self.update_sealing_key_series(slot);
        if let Err(e) = self.validate_tickets(new_epoch, tickets)? {
            *self = prev_state;
            return Err(e);
        }

        let markers = (
            self.collect_epoch_marker(new_epoch),
            self.collect_tickets_marker(slot),
        );

        self.tau = slot;
        Ok(markers)
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
        self.gamma_k = self
            .iota
            .clone()
            .into_iter()
            .map(|validator| {
                if self.post_offenders.contains(&validator.ed25519) {
                    Default::default()
                } else {
                    validator
                }
            })
            .collect();

        // update bandersnatch ring commitment
        let keys = self
            .gamma_k
            .iter()
            .map(|validator| validator.bandersnatch)
            .collect::<Vec<_>>();
        self.gamma_z = crypto::ring::commitment(keys);

        // TODO: graypaper reference: 6.14
    }

    /// Verifies tickets and updates the accumulator according to graypaper section 6.7.
    pub fn validate_tickets(
        &mut self,
        new_epoch: bool,
        tickets: &TicketsExtrinsic,
    ) -> Result<Result<()>> {
        let verifier =
            crypto::ring::verifier(self.gamma_k.iter().map(|v| v.bandersnatch).collect());

        let mut new_tickets = Vec::new();

        // Process each ticket envelope
        for envelope in tickets {
            // 1. Verify ticket attempt
            //
            // graypaper reference: 6.29
            if envelope.attempt > score::TICKET_ENTRIES_PER_VALIDATOR {
                return Ok(Err(Error::BadTicketAttempt));
            }

            // 2. Construct ring VRF input data according to graypaper 6.7
            //
            // graypaper formula: 6.29
            // X_T ∥ η'_2 ∥ r
            let input_data = [
                b"jam_ticket_seal",              // X_T token
                self.eta[2].as_slice(),          // η'_2 (second-oldest entropy)
                &envelope.attempt.to_le_bytes(), // r (attempt number)
            ]
            .concat();

            // 3. Verify ring VRF signature and get ticket identifier
            let id = match verifier.ring_vrf_verify(
                &input_data, // message data
                &[],         // transcript (empty in this case)
                &envelope.signature,
            ) {
                Ok(id) => id,
                Err(_) => return Ok(Err(Error::BadTicketProof)),
            };

            // 4. Store ticket for accumulation
            new_tickets.push(TicketBody {
                id,
                attempt: envelope.attempt,
            });
        }

        // Check for duplicates
        if self.gamma_a.iter().any(|t| new_tickets.contains(t)) {
            return Ok(Err(Error::DuplicateTicket));
        }

        // Check for bad order
        //
        // graypaper reference: 6.32 & 6.33
        let mut sorted_new_tickets = new_tickets.clone();
        sorted_new_tickets.sort_by(|a, b| a.id.cmp(&b.id));
        if sorted_new_tickets != new_tickets {
            return Ok(Err(Error::BadTicketOrder));
        }

        // Clear the accumulator if we're starting a new epoch
        //
        // graypaper reference: 6.34
        if new_epoch {
            self.gamma_a.clear();
        }

        // Create merged set of tickets (formula 6.35: n ∪ γ_a)
        if new_epoch {
            // New epoch: only use new tickets
            self.gamma_a = new_tickets;
        } else {
            // Same epoch: merge with existing tickets
            self.gamma_a.extend(new_tickets);
        };

        // Sort by identifier
        self.gamma_a.sort_by(|a, b| a.id.cmp(&b.id));

        // Take only the first E tickets (formula 6.35: truncate to E)
        self.gamma_a.truncate(score::EPOCH_LENGTH as usize);

        Ok(Ok(()))
    }

    /// Updates the entropy accumulator.
    ///
    /// graypaper reference: 6.4
    pub fn update_eta(&mut self, new_epoch: bool, entropy: OpaqueHash) {
        // graypaper reference: 6.23
        //
        // eta'_e = H(eta_e || eta'_(e-1))
        if new_epoch {
            let historical_eta = self.eta;
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

    pub fn sequence_tickets(&self) -> Vec<TicketBody> {
        let mut ordered_tickets = Vec::with_capacity(self.gamma_a.len());
        let mid = self.gamma_a.len() / 2;

        for i in 0..mid {
            ordered_tickets.push(self.gamma_a[i]);
            if i + mid < self.gamma_a.len() {
                ordered_tickets.push(self.gamma_a[self.gamma_a.len() - 1 - i]);
            }
        }

        ordered_tickets
    }

    /// Calculates the tickets marker according to graypaper formula 6.28.
    pub fn collect_tickets_marker(&mut self, slot: u32) -> Option<TicketsMark> {
        let curr_epoch = slot / score::EPOCH_LENGTH;
        let prev_epoch = self.tau / score::EPOCH_LENGTH;
        let curr_slot_phase = slot % score::EPOCH_LENGTH;
        let prev_slot_phase = self.tau % score::EPOCH_LENGTH;

        // Return None if:
        // 1. Different epochs (e' ≠ e)
        // 2. Previous slot not before submission period (m ≥ Y)
        // 3. Current slot not after submission period (m' < Y)
        // 4. Accumulator not full (|gamma_a| ≠ E)
        if curr_epoch != prev_epoch
            || prev_slot_phase >= score::CONTEST_DURATION
            || curr_slot_phase < score::CONTEST_DURATION
            || self.gamma_a.len() != score::EPOCH_LENGTH as usize
        {
            return None;
        }

        // Apply Z function to gamma_a (outside-in sequencing)
        let mut tickets = [TicketBody::default(); score::EPOCH_LENGTH as usize];
        tickets.copy_from_slice(&self.sequence_tickets());
        Some(tickets)
    }

    /// Updates the sealing key series according to graypaper formula 6.24.
    pub fn update_sealing_key_series(&mut self, slot: u32) {
        let curr_epoch = slot / score::EPOCH_LENGTH;
        let prev_epoch = self.tau / score::EPOCH_LENGTH;
        let prev_slot_phase = self.tau % score::EPOCH_LENGTH;

        // Case 1: Z(γ_a) when e' = e + 1 ∧ m ≥ Y ∧ |γ_a| = E
        if curr_epoch == prev_epoch + 1
            && prev_slot_phase >= score::CONTEST_DURATION
            && self.gamma_a.len() == score::EPOCH_LENGTH as usize
        {
            // Apply Z function (outside-in sequencing)
            self.gamma_s = TicketsOrKeys::Tickets(self.sequence_tickets());
        }
        // Case 2: Keep existing γ_s when e' = e
        else if curr_epoch == prev_epoch {
            // No change needed to gamma_s
        }
        // Case 3: Use fallback key sequence F(η'_2, κ') otherwise
        else {
            // Generate fallback key sequence using η'_2 and κ'
            let mut fallback_keys = Vec::with_capacity(score::EPOCH_LENGTH as usize);

            for i in 0..score::EPOCH_LENGTH {
                // Construct input for hash: η'_2 ∥ E_4(i)
                let input = [self.eta[2].as_slice(), &i.to_le_bytes()].concat();

                // Hash input to get validator index
                let hash = crypto::blake2b(&input);
                let index =
                    u32::from_le_bytes(hash[0..4].try_into().unwrap()) % (self.kappa.len() as u32);

                // Get validator's Bandersnatch key
                fallback_keys.push(self.kappa[index as usize].bandersnatch);
            }

            self.gamma_s = TicketsOrKeys::Keys(fallback_keys);
        }
    }
}
