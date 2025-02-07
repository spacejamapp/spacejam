//! Block builder

use crate::{
    block::BlockInfo, extrinsic::TicketsOrKeys, state::Storage, validator::Validator, Block,
    Extrinsic,
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
    pub fn seal(mut self, validator: &impl Validator, db: &impl Storage) -> anyhow::Result<Self> {
        let keys: Vec<[u8; 32]> = db
            .current_validators()?
            .unwrap_or_default()
            .into_iter()
            .map(|v| v.bandersnatch)
            .collect();

        let entropy = db.entropy()?.unwrap_or_default();
        let safrole = db.safrole()?.unwrap_or_default();
        let message = codec::encode(&self.0)?;
        self.header.seal = match safrole.series {
            TicketsOrKeys::Tickets(tickets) => {
                let entry_index = tickets
                    .iter()
                    .enumerate()
                    .find(|(_, t)| t.attempt as u32 == self.header.slot)
                    .map(|(i, _)| i)
                    .unwrap_or_default();
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

        self.header.entropy_source = {
            let mut context = crate::JAM_ENTROPY.to_vec();
            context.extend_from_slice(&validator.bandersnatch_output(&self.header.seal)?);
            validator.bandersnatch_sign(&keys, &context, &[])?
        };

        Ok(self)
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
