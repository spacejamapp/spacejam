//! Block header utilities

use score::{
    block::Header,
    extrinsic::{TicketBody, TicketsOrKeys},
    safrole::{ValidatorIter, ValidatorsData},
};
use std::sync::Arc;

/// Validate the header
pub fn validate(
    header: &Header,
    new_epoch: bool,
    validators: &ValidatorsData,
    entropy: score::EntropyBuffer,
    safrole: &score::safrole::Safrole,
    verifier: Arc<crypto::vrf::Verifier>,
) -> anyhow::Result<()> {
    let slot = (header.slot % score::EPOCH_LENGTH) as usize;
    let entropy_buffer = entropy;
    let mut ticket = None;
    let entropy = if new_epoch {
        entropy_buffer[2]
    } else {
        entropy_buffer[3]
    };

    // check the ticket mark
    if new_epoch && safrole.accumulator.len() == score::EPOCH_LENGTH as usize {
        let mut tickets = [TicketBody::default(); score::EPOCH_LENGTH as usize];
        tickets.copy_from_slice(&TicketBody::sequence(&safrole.accumulator));
        ticket = Some(tickets[slot]);
    } else if let TicketsOrKeys::Tickets(tickets) = safrole.series {
        ticket = Some(tickets[slot]);
    }

    // if in fallback, check the author index
    //
    // FIXME: this should be cached in production, embed this here for
    // the workaround of the fuzz tests.
    if ticket.is_none() {
        let keys = if new_epoch {
            let TicketsOrKeys::Keys(keys) =
                TicketsOrKeys::fallback(validators.bandersnatch(), entropy_buffer[1])
            else {
                anyhow::bail!("invalid series");
            };
            keys
        } else {
            let TicketsOrKeys::Keys(keys) = safrole.series else {
                anyhow::bail!("invalid series");
            };
            keys
        };

        let vals = if new_epoch {
            safrole.validators.bandersnatch()
        } else {
            validators.bandersnatch()
        };

        if header.author_index as usize >= score::VALIDATORS_COUNT as usize {
            anyhow::bail!("invalid block author index");
        }

        if keys[slot] != vals[header.author_index as usize] {
            anyhow::bail!("invalid block author");
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

    if let Some(ticket) = ticket
        && ticket.id != ticket_output?
    {
        anyhow::bail!("header seal mismatched");
    }

    entropy_output
}
