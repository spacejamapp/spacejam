//! state transition traces

use score::{block::BlockJson, Block, OpaqueHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// State transition trace input
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestInput {
    /// The state
    #[json(nested)]
    pub pre_state: State,

    /// The block
    #[json(nested)]
    pub block: Block,
}

/// State transition trace output
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct TestOutput {
    /// The post-state
    #[json(nested)]
    pub post_state: State,
}

/// State transition trace state
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct State {
    /// The state root
    #[json(hex)]
    pub state_root: OpaqueHash,

    /// The key-values
    #[json(nested)]
    pub keyvals: Vec<KeyValue>,
}

/// State transition trace key-value
#[derive(Debug, Serialize, Deserialize, Json)]
pub struct KeyValue {
    /// The key
    #[json(hex)]
    pub key: Vec<u8>,

    /// The value
    #[json(hex)]
    pub value: Vec<u8>,
}

mod fallback {
    include!(concat!(env!("OUT_DIR"), "/traces_fallback.rs"));
}

mod safrole {
    include!(concat!(env!("OUT_DIR"), "/traces_safrole.rs"));
}

// TODO: with work report integration
//
/* mod reports_l0 {
    include!(concat!(env!("OUT_DIR"), "/traces_reports_l0.rs"));
} */

/// importer tests
pub mod importer {
    use runtime::{storage::SyncStorage, Storage};
    use score::{
        block::Header,
        extrinsic::{TicketBody, TicketsOrKeys},
        safrole::ValidatorIter,
    };

    /// Validate a header
    pub fn validate(header: &Header, storage: &impl Storage) -> anyhow::Result<()> {
        let slot = storage.timeslot()?;
        let local_epoch = slot / score::EPOCH_LENGTH;
        let remote_epoch = header.slot / score::EPOCH_LENGTH;
        let new_epoch = remote_epoch > local_epoch;

        // select the entropy
        let entropy_buffer = storage.entropy()?;
        let entropy = if header.epoch_mark.is_some() {
            entropy_buffer[2]
        } else {
            entropy_buffer[3]
        };
        let mut ticket = None;

        // check the ticket mark
        let slot = (header.slot % score::EPOCH_LENGTH) as usize;
        if new_epoch {
            if let Ok(tickets) = storage.next_series() {
                ticket = Some(tickets[slot]);
            }
        } else if let Ok(TicketsOrKeys::Tickets(tickets)) = storage.series() {
            ticket = Some(tickets[slot as usize]);
        }

        // indicate the keys to be used
        let keys = if new_epoch {
            storage.next_validators()?
        } else {
            storage.current_validators()?
        }
        .bandersnatch();

        // construct the message
        let context = codec::encode(&header)?;
        let context = context[..context.len() - 96].to_vec();

        // construct the context
        let mut message = Vec::new();
        if let Some(ticket) = ticket {
            tracing::trace!(
                "using ticket#{}@0x{}",
                ticket.attempt,
                hex::encode(ticket.id)
            );
            message = TicketBody::message(ticket.attempt, &entropy);
        } else {
            message.extend_from_slice(&score::JAM_FALLBACK_SEAL);
            message.extend_from_slice(&entropy);
        }

        // check the ticket seal
        if let Some(ticket) = ticket {
            tracing::trace!(
                "[safrole] verifying header seal with entropy: 0x{}, using ticket#{}@0x{}, author_index: {}",
                hex::encode(entropy.as_ref()),
                ticket.attempt,
                hex::encode(ticket.id),
                header.author_index
            );
        } else {
            tracing::trace!(
                "[fallback] verifying header seal with entropy: 0x{}",
                hex::encode(entropy.as_ref())
            );
        }
        let author_index = header.author_index;
        let verifier = crypto::ring::verifier(keys.clone());
        let output = verifier
            .ietf_vrf_verify(&message, &context, &header.seal, author_index as usize)
            .map_err(|e| anyhow::anyhow!("ticket seal verification failed: {}", e))?;

        tracing::trace!("vrf verification output: 0x{}", hex::encode(output));
        if let Some(ticket) = ticket {
            if ticket.id != output {
                let TicketsOrKeys::Tickets(tickets) = storage.series()? else {
                    anyhow::bail!("ticket series not found");
                };
                tracing::error!(
                    "ticket series: {:?}",
                    tickets.into_iter().map(|t| t.id).collect::<Vec<_>>()
                );
                anyhow::bail!("header seal mismatched");
            }
        }

        // verify entropy source
        verifier
            .ietf_vrf_verify(
                &[
                    score::JAM_ENTROPY.as_slice(),
                    crypto::vrf::ietf_output(header.seal)?.as_slice(),
                ]
                .concat(),
                &[],
                &header.entropy_source,
                author_index as usize,
            )
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("entropy source verification failed: {}", e))?;

        Ok(())
    }
}
