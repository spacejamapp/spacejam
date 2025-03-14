//! Block builder

use crate::{
    block::BlockInfo, extrinsic::TicketsOrKeys, runtime::Validator, BandersnatchPublic, Block,
    EntropyBuffer, Extrinsic, TimeSlot,
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
        let message = codec::encode(&self.header)?;
        self.header.seal = match series {
            TicketsOrKeys::Tickets(tickets) => {
                let slot_in_epoch = self.header.slot % crate::EPOCH_LENGTH;
                let tickets_count = tickets.len() as u32;
                let entry_index = if slot_in_epoch < tickets_count / 2 {
                    slot_in_epoch
                } else {
                    // TODO: safe math?
                    tickets_count - 1 - (slot_in_epoch - tickets_count / 2)
                };
                let mut context = crate::JAM_TICKET_SEAL.to_vec();
                context.extend_from_slice(&entropy[3]);
                context.push(entry_index as u8);
                validator.bandersnatch_sign(&keys, &context, &message)?
            }
            TicketsOrKeys::Keys(keys) => {
                let mut context = crate::JAM_FALLBACK_SEAL.to_vec();
                context.extend_from_slice(&entropy[3]);
                validator.bandersnatch_sign(&keys, &context, &message)?
            }
        };

        // set the entropy source
        self.header.entropy_source = {
            let mut context = crate::JAM_ENTROPY.to_vec();
            context.extend_from_slice(&validator.bandersnatch_output(&self.header.seal)?);
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
