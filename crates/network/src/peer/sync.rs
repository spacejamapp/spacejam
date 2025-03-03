//! Sync information.

use anyhow::Result;
use score::{runtime::Head, OpaqueHash, TimeSlot};

/// Sync information.
#[derive(Clone, Default)]
pub struct Sync {
    /// The finalized head.
    pub head: Head,

    /// The leaves.
    pub leaves: Vec<Head>,
}

impl Sync {
    /// Create a new sync information from the receiver stream.
    pub async fn read(recv: &mut quinn::RecvStream) -> Result<Self> {
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
}
