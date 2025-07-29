//! Segment storage for context

use score::{OpaqueHash, Segment};
use std::collections::BTreeMap;
use tokio::sync::RwLock;

/// In-memory segment storage for development
#[derive(Default)]
pub struct SegmentStorage {
    /// Map from segment root to segments
    segments: RwLock<BTreeMap<OpaqueHash, Vec<Segment>>>,
}

impl SegmentStorage {
    /// Create a new segment storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Store segments under a root hash
    pub async fn store_segments(&self, root: OpaqueHash, segments: Vec<Segment>) {
        let mut storage = self.segments.write().await;
        storage.insert(root, segments);
    }

    /// Retrieve segments by root hash
    pub async fn get_segments(&self, root: &OpaqueHash) -> Option<Vec<Segment>> {
        let storage = self.segments.read().await;
        storage.get(root).cloned()
    }

    /// Get a specific segment by root and index
    pub async fn get_segment(&self, root: &OpaqueHash, index: u16) -> Option<Segment> {
        let storage = self.segments.read().await;
        storage.get(root)?.get(index as usize).copied()
    }
}
