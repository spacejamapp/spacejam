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
pub struct Handshake {
    /// The finalized head.
    pub head: Head,

    /// The leaves.
    pub leaves: Vec<Head>,
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
        let mut leaves = Vec::new();
        let leaves_len = u32::from_le_bytes(len) as usize * 32;
        for _ in 0..leaves_len {
            let mut hash = [0; 32];
            recv.read(&mut hash).await?;
            let mut slot = [0; 4];
            recv.read(&mut slot).await?;
            leaves.push(Head {
                hash: OpaqueHash::from(hash),
                slot: TimeSlot::from_le_bytes(slot),
            });
        }

        Ok(Self { head, leaves })
    }

    /// Verify the sync information.
    ///
    /// Mainly for verifying if the remote peer is on the same chain.
    pub async fn verify<C: Config>(
        &self,
        network: &Network<C>,
    ) -> anyhow::Result<Option<ce128::Request>> {
        let finalized = network.storage.get_finalized()?;

        // verify if the remote peer is on the same chain.
        if finalized.slot > self.head.slot {
            let hash = network.storage.get_hash(self.head.slot)?;
            if hash != self.head.hash {
                anyhow::bail!("head hash mismatched");
            }
        }

        // append the leaves to grandpa
        if finalized.hash == self.head.hash {
            let grandpa = network.runtime.grandpa.read().await.clone();
            let gptr = network.runtime.grandpa.clone();

            let leaves = self
                .leaves
                .iter()
                .filter(|leaf| !grandpa.leaves.contains_key(leaf))
                .map(|leaf| (leaf.clone(), HashSet::new()));

            gptr.write().await.leaves.extend(leaves);
        }

        // request the missing blocks
        if finalized.slot < self.head.slot {
            let request = ce128::Request {
                hash: finalized.hash,
                direction: 0,
                maximum: self.head.slot - finalized.slot,
            };
            return Ok(Some(request));
        }

        Ok(None)
    }
}
