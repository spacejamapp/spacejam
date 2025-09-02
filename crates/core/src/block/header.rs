//! Block header

use crate::safrole::ValidatorData;
use crate::EntropyBuffer;
use crate::EPOCH_LENGTH;
use crate::VALIDATORS_COUNT;
use crate::{
    extrinsic::*, BandersnatchPublic, BandersnatchVrfSignature, Ed25519Public, Entropy, HeaderHash,
    OpaqueHash, StateRoot, TimeSlot, ValidatorIndex,
};

use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents the tickets mark in a block header.
pub type TicketsMark = [TicketBody; EPOCH_LENGTH as usize];

/// Represents the epoch mark in a block header.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct EpochMark {
    /// The entropy
    #[json(hex)]
    pub entropy: Entropy,

    /// The tickets entropy
    #[json(hex)]
    pub tickets_entropy: Entropy,

    /// The validators
    #[json(Vec<EValidatorJson>)]
    pub validators: [EValidator; VALIDATORS_COUNT as usize],
}

/// Represents the epoch validator in a block header.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default, Copy)]
pub struct EValidator {
    /// The bandersnatch public key
    #[json(hex)]
    pub bandersnatch: BandersnatchPublic,

    /// The ed25519 public key
    #[json(hex)]
    pub ed25519: Ed25519Public,
}

/// Represents the header of a block.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct Header {
    /// The parent block hash (H_p)
    #[json(hex)]
    pub parent: HeaderHash,

    /// The parent state root (H_r)
    #[json(hex)]
    pub parent_state_root: StateRoot,

    /// The extrinsic hash (H_x)
    #[json(hex)]
    pub extrinsic_hash: OpaqueHash,

    /// The slot of the block (H_t)
    pub slot: TimeSlot,

    /// The epoch mark (H_e)
    ///
    /// This will be some if new epoch is started.
    #[json(nested)]
    pub epoch_mark: Option<EpochMark>,

    /// The winning tickets marker (H_w)
    ///
    /// This will be some at the end of ticket submission period.
    #[json(Option<Vec<TicketBodyJson>>)]
    pub tickets_mark: Option<TicketsMark>,

    /// The author index (H_i)
    pub author_index: ValidatorIndex,

    /// The entropy source (H_v)
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub entropy_source: BandersnatchVrfSignature,

    /// The offenders mark (H_o)
    #[json(hex)]
    pub offenders_mark: Vec<Ed25519Public>,

    /// The seal (H_s)
    #[json(hex)]
    #[serde(with = "codec::bytes")]
    pub seal: BandersnatchVrfSignature,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            parent: HeaderHash::default(),
            parent_state_root: StateRoot::default(),
            extrinsic_hash: OpaqueHash::default(),
            slot: TimeSlot::default(),
            epoch_mark: None,
            tickets_mark: None,
            offenders_mark: vec![],
            author_index: ValidatorIndex::default(),
            entropy_source: [0; 96],
            seal: [0; 96],
        }
    }
}

impl Header {
    /// Validate the header
    pub fn validate(
        &self,
        new_epoch: bool,
        entropy: EntropyBuffer,
        next: &[ValidatorData],
        current: &[ValidatorData],
        series: &TicketsOrKeys,
    ) -> anyhow::Result<()> {
        let slot = (self.slot % crate::EPOCH_LENGTH) as usize;
        let entropy_buffer = entropy;
        let mut ticket = None;
        let entropy = if new_epoch {
            entropy_buffer[2]
        } else {
            entropy_buffer[3]
        };

        // check the ticket mark
        if let TicketsOrKeys::Tickets(tickets) = series {
            ticket = Some(tickets[slot]);
        }

        // indicate the keys to be used
        let keys = if new_epoch { next } else { current }
            .iter()
            .map(|v| v.bandersnatch)
            .collect::<Vec<_>>();

        // construct the message
        let encoded = codec::encode(&self)?;
        let context = encoded[..encoded.len() - 96].to_vec();

        // construct the context
        let mut message = Vec::new();
        if let Some(ticket) = ticket {
            message = TicketBody::message(ticket.attempt, &entropy);
        } else {
            message.extend_from_slice(&crate::JAM_FALLBACK_SEAL);
            message.extend_from_slice(&entropy);
        }

        // check the ticket seal
        let author_index = self.author_index;
        let verifier = crypto::ring::verifier(keys.clone());
        let output = verifier
            .ietf_vrf_verify(&message, &context, &self.seal, author_index as usize)
            .map_err(|e| {
                anyhow::anyhow!("ticket seal verification failed: {e}, new_epoch={new_epoch}")
            })?;

        if let Some(ticket) = ticket {
            if ticket.id != output {
                anyhow::bail!("header seal mismatched");
            }
        }

        // verify entropy source
        let extracted_vrf_output = crypto::vrf::ietf_output(self.seal)?;
        let entropy_message = [&crate::JAM_ENTROPY[..], &extracted_vrf_output[..]].concat();
        verifier
            .ietf_vrf_verify(
                &entropy_message,
                &[],
                &self.entropy_source,
                author_index as usize,
            )
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("entropy source verification failed: {}", e))?;

        Ok(())
    }
}

/// The head of the chain
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Json)]
pub struct Head {
    /// The hash of the head of the chain.
    #[json(hex)]
    pub hash: OpaqueHash,

    /// The slot of this head.
    pub slot: TimeSlot,
}

impl Ord for Head {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.slot.cmp(&other.slot)
    }
}

impl PartialOrd for Head {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "crypto")]
mod crypto_impl {
    use super::*;

    impl Header {
        /// Get the hash of the header
        pub fn hash(&self) -> anyhow::Result<HeaderHash> {
            Ok(crypto::blake2b(&codec::encode(self)?))
        }

        /// Get the head of the header
        pub fn head(&self) -> anyhow::Result<Head> {
            Ok(Head {
                hash: self.hash()?,
                slot: self.slot,
            })
        }
    }
}
