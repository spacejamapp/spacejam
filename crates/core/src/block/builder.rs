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

        self.header.seal = match series {
            TicketsOrKeys::Tickets(tickets) => {
                let slot = (self.header.slot % crate::EPOCH_LENGTH) as usize;
                let ticket = tickets[slot];
                let message = TicketBody::message(ticket.attempt, &entropy[3]);
                validator.bandersnatch_sign(keys, &context, &message)?
            }
            TicketsOrKeys::Keys(_) => {
                let mut message = crate::JAM_FALLBACK_SEAL.to_vec();
                message.extend_from_slice(&entropy[3]);
                validator.bandersnatch_sign(keys, &context, &message)?
            }
        };

        // set the entropy source
        self.header.entropy_source = {
            let mut context = crate::JAM_ENTROPY.to_vec();
            context.extend_from_slice(&crypto::vrf::ietf_output(self.header.seal)?);
            validator.bandersnatch_sign(keys, &context, &[])?
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
