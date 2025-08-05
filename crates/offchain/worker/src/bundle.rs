//! Work package bundle

use anyhow::Result;
use score::{
    service::{WorkPackage, WorkPackageSpec},
    OpaqueHash, Segment,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::d3l::{Justification, PageProof};

/// Work package bundle
#[derive(Serialize, Deserialize)]
pub struct WorkPackageBundle {
    /// The work package itself
    pub package: WorkPackage,

    /// The extrinsic data
    ///
    /// TODO: Vec instead of Map?
    pub extrinsic: HashMap<OpaqueHash, Vec<u8>>,

    /// The concatenated import segments along with their proofs of correctnes
    pub imports_with_proofs: Vec<(Vec<u8>, Vec<Justification>)>,
}

impl WorkPackageBundle {
    /// Create a new work package bundle with default empty collections
    pub fn new(package: score::service::WorkPackage) -> Self {
        Self {
            package,
            extrinsic: Default::default(),
            imports_with_proofs: Default::default(),
        }
    }

    /// The availability specifier function A
    pub async fn specify(&self, exported: Vec<Segment>) -> Result<WorkPackageSpec> {
        // 1. Generate segment chunks (s♣)
        let exports_root = crypto::blake2b(
            exported
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let proofs = PageProof::proofs(&exported, &exports_root).await?;
        let encoded = codec::encode(&(
            exported.to_vec().iter().flatten().collect::<Vec<_>>(),
            proofs,
        ))?;
        let exported_chunks = erasure::encode(encoded).await?;
        let exported_chunk_hashes: Vec<OpaqueHash> = exported_chunks
            .iter()
            .map(|chunk| crypto::blake2b(chunk))
            .collect();

        // 2. Generate bundle chunks (s♣)
        let bundle = codec::encode(self)?;
        let length = bundle.len() as u32;
        let bundle_chunks = erasure::encode(bundle).await?;
        let bundle_chunk_hashes: Vec<OpaqueHash> = bundle_chunks
            .iter()
            .map(|chunk| crypto::blake2b(chunk))
            .collect();

        // 3. get merkle root of all chunks
        let chunks = bundle_chunk_hashes.len().max(exported_chunk_hashes.len());
        let leaves: Vec<Vec<u8>> = (0..chunks)
            .map(|i| {
                let mut leaf = Vec::new();
                if let Some(bundle_chunk) = bundle_chunk_hashes.get(i) {
                    leaf.extend_from_slice(bundle_chunk);
                }
                if let Some(exported_chunk) = exported_chunk_hashes.get(i) {
                    leaf.extend_from_slice(exported_chunk);
                }
                leaf
            })
            .collect();

        Ok(WorkPackageSpec {
            hash: crypto::blake2b(&codec::encode(&self.package)?),
            length,
            erasure_root: crypto::merkle::root(&leaves),
            exports_root: crypto::blake2b(
                exported_chunk_hashes
                    .iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            exports_count: exported.len() as u16,
        })
    }
}
