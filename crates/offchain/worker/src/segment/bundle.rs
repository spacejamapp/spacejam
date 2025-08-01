//! Segment bundle operations

use crate::segment::Justification;
use score::{service::WorkPackage, OpaqueHash, Segment};
use serde::{Deserialize, Serialize};

/// Bundle components for erasure root computation per Gray Paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentBundle {
    /// The work package
    pub package: WorkPackage,

    /// The extrinsics
    pub extrinsics: Vec<Vec<u8>>,

    /// The imports
    #[serde(with = "codec::bytes::array")]
    pub imports: Vec<Segment>,

    /// The justifications for imported segments per Gray Paper specification
    pub justifications: Vec<Justification>,
}

impl SegmentBundle {
    /// Compute erasure root
    pub fn erasure_root(&self, exports: &[Segment]) -> anyhow::Result<OpaqueHash> {
        let encoded = codec::encode(self)?;
        let bundle_chunks = erasure::encode_sync(encoded)?;
        let segment_chunks = if exports.is_empty() {
            vec![]
        } else {
            let all_segments: Vec<u8> = exports.iter().flat_map(|s| s.iter()).copied().collect();
            erasure::encode_sync(all_segments)?
        };

        let transposed = transpose_chunks(&bundle_chunks, &segment_chunks);
        Ok(crypto::merkle::root(&transposed))
    }
}

/// Transpose chunks matrix for erasure root computation
fn transpose_chunks(bundle_chunks: &[Vec<u8>], segment_chunks: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let max_chunks = bundle_chunks.len().max(segment_chunks.len());

    (0..max_chunks)
        .map(|i| {
            [bundle_chunks.get(i), segment_chunks.get(i)]
                .into_iter()
                .flatten()
                .flat_map(|chunk| chunk.iter())
                .copied()
                .collect()
        })
        .collect()
}
