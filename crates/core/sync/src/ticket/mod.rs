//! Spacejam's SAFRole prototype

pub use error::{Error, Result};
use score::{
    extrinsic::{
        ticket::{TicketBody, TicketsExtrinsic, TicketsOrKeys},
        TicketsAccumulator,
    },
    safrole::Safrole,
    validator::{ValidatorData, Validators, ValidatorsData},
    Ed25519Public, OpaqueHash,
};

pub mod error;

/// (η') Updates the entropy accumulator.
///
/// graypaper reference: 6.4
pub fn eta(new_epoch: bool, eta: &[OpaqueHash; 4], entropy: OpaqueHash) -> [OpaqueHash; 4] {
    let mut next = *eta;

    // (6.23) eta'_e = H(eta_e || eta'_(e-1))
    if new_epoch {
        let historical_eta = eta;
        next[1..].copy_from_slice(&historical_eta[..3]);
    }

    // (6.22) eta'_0 = H(eta_0 || Y(H_v))
    let eta_0 = crypto::blake2b(&[eta[0], entropy].concat());
    next[0] = eta_0;
    next
}

/// (ι', κ', λ') Returns the next state of validators.
pub fn validators(new_epoch: bool, next: &ValidatorsData, validators: &Validators) -> Validators {
    let mut validators = validators.clone();
    if !new_epoch {
        return validators;
    }

    validators.previous = validators.previous(new_epoch);
    validators.current = validators.current(new_epoch, next);
    validators
}

/// (γ') Enacts an epoch change and updates the entropy accumulator.
pub fn safrole(
    tau: u32,
    slot: u32,
    entropy: [OpaqueHash; 4],
    offenders: &[Ed25519Public],
    safrole: &Safrole,
    validators: &Validators,
    tickets: &TicketsExtrinsic,
) -> Result<Safrole> {
    if slot <= tau {
        return Err(Error::BadSlot);
    }

    if slot % score::CONTEST_DURATION == 0 && !tickets.is_empty() {
        return Err(Error::UnexpectedTicket);
    }

    let epoch = slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (tau / score::EPOCH_LENGTH);
    let mut safrole = safrole.clone();
    safrole.series = sealing_key_series(tau, slot, entropy, &safrole, &validators.current);
    safrole.accumulator = accumulator(
        new_epoch,
        &safrole.accumulator,
        entropy,
        &safrole.validators,
        tickets,
    )?;
    safrole.validators = safrole.next(new_epoch, &validators.next, offenders);
    safrole.ring_commitment = safrole.commitment(new_epoch);
    Ok(safrole)
}

/// (γ_a') Verifies tickets and updates the accumulator according to graypaper section 6.7.
pub fn accumulator(
    new_epoch: bool,
    accumulator: &TicketsAccumulator,
    entropy: [OpaqueHash; 4],
    next: &[ValidatorData],
    tickets: &TicketsExtrinsic,
) -> Result<TicketsAccumulator> {
    // NOTE: gamma_k has already been updated at this point
    //
    // TODO: double check the validator set used for the ring VRF
    let verifier = crypto::ring::verifier(next.iter().map(|v| v.bandersnatch).collect());

    // Process each ticket envelope
    let mut new_tickets = Vec::new();
    for envelope in tickets {
        // 1. Verify ticket attempt (6.29)
        if envelope.attempt > score::TICKET_ENTRIES_PER_VALIDATOR {
            return Err(Error::BadTicketAttempt);
        }

        // 2. Construct ring VRF input data (6.29)
        let input_data = [
            &score::JAM_TICKET_SEAL,         // X_T token
            entropy[2].as_slice(),           // η'_2 (second-oldest entropy)
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
            Err(_) => return Err(Error::BadTicketProof),
        };

        // 4. Store ticket for accumulation
        new_tickets.push(TicketBody {
            id,
            attempt: envelope.attempt,
        });
    }

    // Check for bad order: 6.32 & 6.33
    let mut sorted_new_tickets = new_tickets.clone();
    sorted_new_tickets.sort_by(|a, b| a.id.cmp(&b.id));
    if sorted_new_tickets != new_tickets {
        return Err(Error::BadTicketOrder);
    }

    // Check for duplicates
    let mut accumulator = accumulator.clone();
    if accumulator.iter().any(|t| new_tickets.contains(t)) {
        return Err(Error::DuplicateTicket);
    }

    // Clear the accumulator if we're starting a new epoch: 6.34
    if new_epoch {
        accumulator.clear();
    }

    // Create merged set of tickets (formula 6.35: n ∪ γ_a)
    if new_epoch {
        accumulator = new_tickets;
    } else {
        accumulator.extend(new_tickets);
    };

    // Sort by identifier
    //
    // Take only the first E tickets (formula 6.35: truncate to E)
    accumulator.sort_by(|a, b| a.id.cmp(&b.id));
    accumulator.truncate(score::EPOCH_LENGTH as usize);
    Ok(accumulator)
}

/// (γ_s') Updates the sealing key series according to graypaper formula 6.24.
pub fn sealing_key_series(
    tau: u32,
    slot: u32,
    entropy: [OpaqueHash; 4],
    safrole: &Safrole,
    curr_validators: &[ValidatorData],
) -> TicketsOrKeys {
    let mut next = safrole.series.clone();
    let curr_epoch = slot / score::EPOCH_LENGTH;
    let prev_epoch = tau / score::EPOCH_LENGTH;
    let prev_slot_phase = tau % score::EPOCH_LENGTH;

    if curr_epoch == prev_epoch {
        return next;
    }

    if curr_epoch == prev_epoch + 1
        && prev_slot_phase >= score::CONTEST_DURATION
        && safrole.accumulator.len() == score::EPOCH_LENGTH as usize
    {
        next = TicketsOrKeys::Tickets(safrole.tickets());
    } else {
        let mut fallback_keys = Vec::with_capacity(score::EPOCH_LENGTH as usize);
        for i in 0..score::EPOCH_LENGTH {
            let input = [entropy[2].as_slice(), &i.to_le_bytes()].concat();
            let hash = crypto::blake2b(&input);
            let index =
                u32::from_le_bytes(hash[0..4].try_into().unwrap()) % (curr_validators.len() as u32);

            fallback_keys.push(curr_validators[index as usize].bandersnatch);
        }

        next = TicketsOrKeys::Keys(fallback_keys);
    }

    next
}
