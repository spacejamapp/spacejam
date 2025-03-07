//! Handshake handler

use crate::{stream::ce128, Event, Network};
use quinn::{RecvStream, SendStream};
use score::{
    runtime::{storage::BlockStorage, Config, Head, Storage},
    OpaqueHash, TimeSlot,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;

/// Sync information.
#[derive(Debug, Default)]
pub struct Handshake {
    /// The finalized head.
    pub head: Head,

    /// The leaves.
    pub leaves: HashSet<Head>,
}

impl Handshake {
    /// Create a new sync information from the receiver stream.
    pub async fn read(recv: &mut quinn::RecvStream) -> anyhow::Result<Self> {
        // 1. read the finalized hash
        let mut hash = [0; 32];
        recv.read(&mut hash).await?;
        let mut slot = [0; 4];
        recv.read(&mut slot).await?;
        let head = Head {
            hash: OpaqueHash::from(hash),
            slot: TimeSlot::from_le_bytes(slot),
        };

        // 2. read the leaves len
        let mut len = [0; 4];
        recv.read(&mut len).await?;

        // 3. read the leaves
        let mut leaves = HashSet::new();
        let leaves_len = u32::from_le_bytes(len) as usize * 32;
        for _ in 0..leaves_len {
            let mut hash = [0; 32];
            recv.read(&mut hash).await?;
            let mut slot = [0; 4];
            recv.read(&mut slot).await?;
            leaves.insert(Head {
                hash: OpaqueHash::from(hash),
                slot: TimeSlot::from_le_bytes(slot),
            });
        }

        Ok(Self { head, leaves })
    }
}
