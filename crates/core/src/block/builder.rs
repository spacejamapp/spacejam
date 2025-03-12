//! Block builder

use crate::{
    block::BlockInfo,
    extrinsic::TicketsOrKeys,
    runtime::{Storage, Validator},
    Block, Extrinsic,
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
        self.header.slot = crate::block::timeslot()?;
        Ok(self)
    }

    /// Set the extrinsic
    pub fn extrinsic(mut self, extrinsic: Extrinsic) -> anyhow::Result<Self> {
        self.header.extrinsic_hash = extrinsic.hash()?;
        self.extrinsic = extrinsic;
        Ok(self)
    }

    /// Seal the block
    pub fn seal(mut self, validator: &impl Validator, db: &impl Storage) -> anyhow::Result<Block> {
        let keys: Vec<[u8; 32]> = db
            .current_validators()?
            .into_iter()
            .map(|v| v.bandersnatch)
            .collect();

        // 1. set the validator index
        self.header.author_index = keys
            .iter()
            .position(|k| k == &validator.bandersnatch_public_key())
            .ok_or_else(|| anyhow::anyhow!("validator not present in the current validator set"))?
            as u16;

        // 2. set the seal
        let entropy = db.entropy()?;
        let safrole = db.safrole()?;
        let message = codec::encode(&self.0.header)?;
        self.header.seal = match safrole.series {
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
            TicketsOrKeys::Keys(_) => {
                let mut context = crate::JAM_FALLBACK_SEAL.to_vec();
                context.extend_from_slice(&entropy[3]);
                validator.bandersnatch_sign(&keys, &context, &message)?
            }
        };

        // 3. set the entropy source
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
