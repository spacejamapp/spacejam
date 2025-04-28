//! This module contains the block and its related structures.

use crate::{
    extrinsic::*,
    service::{ReportedWorkPackage, ReportedWorkPackageJson},
    Entropy, OpaqueHash, TimeSlot,
};
use header::{EValidator, EpochMark};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::time::{Duration, UNIX_EPOCH};
pub use {
    builder::Builder,
    header::{Head, Header, HeaderJson},
    history::{Mmr, MmrJson},
};

#[cfg(feature = "crypto")]
pub use crypto_impl::*;

mod builder;
pub mod header;
pub mod history;

/// Represents a block in the system.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Default, Clone)]
pub struct Block {
    /// The header of the block
    #[json(nested)]
    pub header: Header,
    /// The extrinsic of the block
    #[json(nested)]
    pub extrinsic: Extrinsic,
}

impl Block {
    /// Returns a builder for the block
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the genesis block
    pub fn genesis(validators: [EValidator; crate::VALIDATORS_COUNT as usize]) -> Self {
        let header = Header {
            epoch_mark: Some(EpochMark {
                entropy: Entropy::default(),
                tickets_entropy: Entropy::default(),
                validators,
            }),
            ..Default::default()
        };
        Self {
            header,
            extrinsic: Extrinsic::default(),
        }
    }
}

/// Returns the current timeslot
pub fn timeslot() -> TimeSlot {
    now() / crate::SLOT_PERIOD
}

/// Returns the current time in seconds
pub fn now() -> u32 {
    let era = Duration::from_secs(crate::JAM_COMMON_ERA_AFTER_UNIX_EPOCH as u64);
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH + era)
        .expect("time went backwards")
        .as_secs() as u32
}

/// Represents information about a block.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct BlockInfo {
    #[json(hex)]
    pub header_hash: OpaqueHash,
    #[json(nested)]
    pub mmr: Mmr,
    #[json(hex)]
    pub state_root: OpaqueHash,
    #[json(nested)]
    pub reported: Vec<ReportedWorkPackage>,
}

#[cfg(feature = "crypto")]
mod crypto_impl {
    pub use super::history::History;
    use super::*;

    impl Block {
        /// Returns the hash of the block
        pub fn hash(&self) -> anyhow::Result<crate::HeaderHash> {
            let encoded = codec::encode(&self.header)?;
            Ok(crypto::blake2b(&encoded))
        }
    }

    impl From<Header> for BlockInfo {
        fn from(header: Header) -> Self {
            Self {
                header_hash: header.hash().unwrap(),
                mmr: Mmr::default(),
                state_root: header.parent_state_root,
                reported: vec![],
            }
        }
    }
}
