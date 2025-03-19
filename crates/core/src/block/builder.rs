//! Block builder

use crate::{
    block::BlockInfo,
    extrinsic::{TicketBody, TicketsOrKeys},
    runtime::Validator,
    BandersnatchPublic, Block, EntropyBuffer, Extrinsic, TimeSlot,
};
use std::ops::{Deref, DerefMut};

/// Block builder
#[derive(Default)]
pub struct Builder(Block);

impl Builder {
    /// Set the parent block
    pub fn parent(mut self, info: &BlockInfo) -> anyhow::Result<Self> {
        self.header.parent = info.header_hash;
        self.header.parent_state_root = info.state_root;
        Ok(self)
    }

    /// Set the extrinsic
    pub fn extrinsic(mut self, extrinsic: Extrinsic) -> anyhow::Result<Self> {
        self.header.extrinsic_hash = extrinsic.hash()?;
        self.extrinsic = extrinsic;
        Ok(self)
    }

    /// Set the timeslot
    pub fn timeslot(mut self, timeslot: TimeSlot) -> Self {
        self.header.slot = timeslot;
        self
    }

    /// Set the author index
    pub fn author_index(mut self, index: u16) -> Self {
        self.header.author_index = index;
        self
    }

    /// Seal the block
    pub fn seal(
        mut self,
        validator: &impl Validator,
        keys: &[BandersnatchPublic],
        series: TicketsOrKeys,
        entropy: EntropyBuffer,
    ) -> anyhow::Result<Block> {
        let context = codec::encode(&self.header)?;
        let mut keys = keys.to_vec();
        let entropy = if let Some(mark) = self.header.epoch_mark.clone() {
            keys = mark.validators.to_vec();
            entropy[2]
        } else {
            entropy[3]
        };

        self.header.seal = match series {
            TicketsOrKeys::Tickets(tickets) => {
                let slot = (self.header.slot % crate::EPOCH_LENGTH) as usize;
                let ticket = tickets[slot];
                tracing::trace!(
                    "sealing block with entropy: 0x{}, ticket#{}@0x{}",
                    hex::encode(entropy.as_ref()),
                    ticket.attempt,
                    hex::encode(ticket.id)
                );
                let message = TicketBody::message(ticket.attempt, &entropy);
                let seal = validator.bandersnatch_sign(&keys, &context, &message)?;

                let verifier = crypto::ring::verifier(keys.clone());
                let output = verifier.ietf_vrf_verify(
                    &message,
                    &context,
                    &seal,
                    self.header.author_index as usize,
                )?;
                if output != ticket.id {
                    tracing::error!(
                        "ticket seal mismatched, expected: 0x{}, got: 0x{}",
                        hex::encode(ticket.id),
                        hex::encode(output)
                    );
                    anyhow::bail!("ticket seal mismatched");
                }

                seal
            }
            TicketsOrKeys::Keys(_) => {
                let mut message = crate::JAM_FALLBACK_SEAL.to_vec();
                message.extend_from_slice(&entropy);
                validator.bandersnatch_sign(&keys, &context, &message)?
            }
        };

        // set the entropy source
        self.header.entropy_source = {
            let mut context = crate::JAM_ENTROPY.to_vec();
            context.extend_from_slice(&crypto::vrf::ietf_output(self.header.seal)?);
            validator.bandersnatch_sign(&keys, &context, &[])?
        };

        Ok(self.into())
    }
}

impl From<Builder> for Block {
    fn from(builder: Builder) -> Self {
        builder.0
    }
}

impl Deref for Builder {
    type Target = Block;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Builder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
