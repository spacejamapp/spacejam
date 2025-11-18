//! Block builder

use crate::{Block, TimeSlot, block::BlockInfo};
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
    #[cfg(feature = "blake2")]
    pub fn extrinsic(mut self, extrinsic: crate::Extrinsic) -> Self {
        self.header.extrinsic_hash = extrinsic.hash();
        self.extrinsic = extrinsic;
        self
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
