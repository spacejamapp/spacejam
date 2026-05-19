//! Block header utilities

use crate::tx::ticket::lazy;
use score::{
    State,
    block::Header,
    extrinsic::{TicketBody, TicketsOrKeys},
    safrole::ValidatorIter,
};

/// Validate the header
pub fn validate(state: State, header: &Header) -> anyhow::Result<()> {
    let new_epoch = header.slot / score::EPOCH_LENGTH > state.timeslot / score::EPOCH_LENGTH;
    self::check(&state, header, new_epoch)?;

    // setup the verifier
    let slot = (header.slot % score::EPOCH_LENGTH) as usize;
    let verifier = if new_epoch {
        lazy::verifier(&state.safrole.validators.bandersnatch())
    } else {
        lazy::verifier(&state.validators.current.bandersnatch())
    };

    // setup the entropy
    let mut ticket = None;
    let entropy = if new_epoch {
        state.entropy[2]
    } else {
        state.entropy[3]
    };

    // check the ticket mark
    if new_epoch && state.safrole.accumulator.len() == score::EPOCH_LENGTH as usize {
        let mut tickets = [TicketBody::default(); score::EPOCH_LENGTH as usize];
        tickets.copy_from_slice(&TicketBody::sequence(&state.safrole.accumulator));
        ticket = Some(tickets[slot]);
    } else if let TicketsOrKeys::Tickets(tickets) = &state.safrole.series
        && !new_epoch
    {
        ticket = Some(tickets[slot]);
    }

    // if in fallback, check the author index
    //
    // FIXME: this should be cached in production, embed this here for
    // the workaround of the fuzz tests.
    if ticket.is_none() {
        let vals = if new_epoch {
            state.safrole.validators.bandersnatch()
        } else {
            state.validators.current.bandersnatch()
        };

        let keys = if new_epoch {
            let TicketsOrKeys::Keys(keys) = TicketsOrKeys::fallback(vals.clone(), state.entropy[1])
            else {
                anyhow::bail!("invalid series");
            };
            keys
        } else {
            let TicketsOrKeys::Keys(keys) = &state.safrole.series else {
                anyhow::bail!("invalid series");
            };
            keys.clone()
        };

        // FIXME: This is a duplicated check for async processing.
        if header.author_index as usize >= score::VALIDATORS_COUNT as usize {
            anyhow::bail!("invalid block author index");
        }

        if keys[slot] != vals[header.author_index as usize] {
            anyhow::bail!(
                "invalid block author, slot={slot}, new_epoch={new_epoch}, author_index={}",
                header.author_index
            );
        }
    }

    // construct the message
    let encoded = codec::encode(&header);
    let context = encoded[..encoded.len() - 96].to_vec();

    // construct the context
    let mut message = Vec::new();
    let mut fallback = false;
    if let Some(ticket) = ticket {
        message = TicketBody::message(ticket.attempt, &entropy);
    } else {
        fallback = true;
        message.extend_from_slice(&score::JAM_FALLBACK_SEAL);
        message.extend_from_slice(&entropy);
    }

    let extracted_vrf_output = crypto::vrf::ietf_output(header.seal)?;
    let entropy_message = [&score::JAM_ENTROPY[..], &extracted_vrf_output[..]].concat();
    let (ticket_output, entropy_output) = rayon::join(
        || {
            verifier
            .ietf_vrf_verify(&message, &context, &header.seal, header.author_index as usize)
            .map_err(|e| {
                anyhow::anyhow!("ticket seal verification failed: {e}, new_epoch={new_epoch}, fallback={fallback}")
            })
        },
        || {
            verifier
                .ietf_vrf_verify(
                    &entropy_message,
                    &[],
                    &header.entropy_source,
                    header.author_index as usize,
                )
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!("entropy source verification failed: {}", e))
        },
    );

    let ticket_output = ticket_output?;
    if let Some(ticket) = ticket
        && ticket.id != ticket_output
    {
        anyhow::bail!("header seal mismatched");
    }

    entropy_output
}

/// Check the marks in the header
pub fn check(state: &State, header: &Header, new_epoch: bool) -> anyhow::Result<()> {
    if header.slot <= state.timeslot {
        anyhow::bail!("block slot is less than or equal to current height");
    }

    if header.author_index >= score::VALIDATORS_COUNT {
        anyhow::bail!("invalid author index");
    }

    // validate the epoch mark
    if let Some(epoch_mark) = &header.epoch_mark {
        let expected = state
            .safrole
            .next(&state.validators.drawn, &header.offenders_mark)
            .evals();
        if epoch_mark.validators[..] != expected[..] {
            anyhow::bail!("epoch mark validators mismatch");
        }

        if epoch_mark.entropy != state.entropy[0] {
            anyhow::bail!("epoch mark entropy mismatch");
        }

        if epoch_mark.tickets_entropy != state.entropy[1] {
            anyhow::bail!("epoch mark tickets entropy mismatch");
        }
    } else if new_epoch {
        anyhow::bail!("epoch mark is required");
    }

    let should_have_tickets_mark = state.safrole.has_tickets_mark(state.timeslot, header.slot);
    if let Some(tickets_mark) = &header.tickets_mark {
        if !should_have_tickets_mark {
            anyhow::bail!("tickets mark present but not expected");
        }

        // Validate content: tickets_mark == Z(accumulator)
        let expected = TicketBody::sequence(&state.safrole.accumulator);
        if tickets_mark[..] != expected[..] {
            anyhow::bail!("tickets mark content mismatch");
        }

        // Validate ticket attempts
        for ticket in tickets_mark {
            if ticket.attempt > score::TICKET_ENTRIES_PER_VALIDATOR as u8 {
                anyhow::bail!("invalid ticket attempt {}", ticket.attempt);
            }
        }
    } else if should_have_tickets_mark {
        anyhow::bail!("tickets mark required but not present");
    }

    // validate the parent header hash
    if let Some(head) = state.recent_blocks.head() {
        if head.header_hash != header.parent {
            anyhow::bail!("parent header hash mismatch");
        }
        if head.state_root != header.parent_state_root {
            anyhow::bail!("parent state root mismatch");
        }
    }

    Ok(())
}
