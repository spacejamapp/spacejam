//! Segment bundle operations per Gray Paper B.4

use crate::d3l::Justification;
use anyhow::Result;
use score::{
    service::{ExtrinsicSpec, WorkPackage, WorkPackageSpec},
    OpaqueHash, Segment,
};
use serde::{Deserialize, Serialize};

/// Bundle specifier
///
/// TODO: taking
///
/// - package hash
/// - an octet sequence of the audit-friendly work-package bundle
/// - the sequence of exported segements
pub struct Specifier {
    /// Components of erasure computation
    bundle: Bundle,

    /// The specification of the work package
    spec: WorkPackageSpec,
}

impl Specifier {
    /// Create a new bundle specifier
    pub fn new(package: WorkPackage) -> Result<Self> {
        let hash = crypto::blake2b(codec::encode(&package)?.as_slice());
        Ok(Self {
            bundle: Bundle {
                package,
                extrinsics: vec![],
                imports: vec![],
                justifications: vec![],
            },
            spec: WorkPackageSpec {
                hash,
                length: 0,
                erasure_root: [0u8; 32],
                exports_root: [0u8; 32],
                exports_count: 0,
            },
        })
    }

    /// Specify the work package with segment export data
    pub fn specify(
        mut self,
        root: OpaqueHash,
        count: u16,
        hashes: Vec<OpaqueHash>,
    ) -> Result<WorkPackageSpec> {
        let bundle = codec::encode(&self.bundle)?;
        self.spec.length = bundle.len() as u32;
        self.spec.erasure_root = self::root(bundle, &hashes)?;
        self.spec.exports_root = root;
        self.spec.exports_count = count;
        Ok(self.spec)
    }

    /// Import a segment
    pub fn import(&mut self, segments: &[Segment]) {
        self.bundle.imports.extend_from_slice(segments);
    }

    /// Add a justification
    pub fn justification(&mut self, justification: Justification) {
        self.bundle.justifications.push(justification);
    }

    /// Add an extrinsic
    pub fn extrinsic(&mut self, spec: &ExtrinsicSpec, extrinsic: &[u8]) -> Result<()> {
        if extrinsic.len() != spec.len as usize {
            return Err(anyhow::anyhow!(
                "Extrinsic data length mismatch for hash {:?}: expected {}, got {}",
                spec.hash,
                spec.len,
                extrinsic.len()
            ));
        }

        self.bundle.extrinsics.push(extrinsic.to_vec());
        Ok(())
    }

    /// Get the hash of the work package
    pub const fn package_hash(&self) -> OpaqueHash {
        self.spec.hash
    }
}

/// Bundle components for erasure root computation per Gray Paper B.4
/// Represents the auditable work-package bundle containing:
/// - Work package itself
/// - Extrinsic data  
/// - Imported segments with justifications
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bundle {
    /// The work package
    pub package: WorkPackage,

    /// The extrinsic data referenced by work items
    pub extrinsics: Vec<Vec<u8>>,

    /// The imported segments (reconstructed from erasure chunks)
    #[serde(with = "codec::bytes::array")]
    pub imports: Vec<Segment>,

    /// The justifications for imported segments per Gray Paper specification  
    pub justifications: Vec<Justification>,
}

/// Compute erasure root per Gray Paper B.4 specification
/// Implements: erasure_root = merkle_root(transpose([b♣, s♣]))
/// Where b♣ = erasure_code(bundle) and s♣ = erasure_code(segments + page_proofs)
pub fn root(bundle: Vec<u8>, segment_chunk_hashes: &[OpaqueHash]) -> Result<OpaqueHash> {
    let bundle_chunks = ::erasure::encode_sync(bundle)?;
    let bundle_chunk_hashes: Vec<OpaqueHash> = bundle_chunks
        .iter()
        .map(|chunk| crypto::blake2b(chunk))
        .collect();

    let transposed_chunks = self::transpose(&bundle_chunk_hashes, segment_chunk_hashes);
    Ok(crypto::merkle::root(&transposed_chunks))
}

/// Transpose chunks matrix for erasure root computation per Gray Paper B.4
/// Combines bundle chunk hashes and segment chunk hashes into transposed matrix
fn transpose(bundle_chunks: &[OpaqueHash], segment_chunks: &[OpaqueHash]) -> Vec<Vec<u8>> {
    let max_chunks = bundle_chunks.len().max(segment_chunks.len());
    (0..max_chunks)
        .map(|i| {
            let mut transposed_chunk = Vec::new();

            // Add bundle chunk hash if available
            if let Some(bundle_chunk) = bundle_chunks.get(i) {
                transposed_chunk.extend_from_slice(bundle_chunk);
            }

            // Add segment chunk hash if available
            if let Some(segment_chunk) = segment_chunks.get(i) {
                transposed_chunk.extend_from_slice(segment_chunk);
            }

            transposed_chunk
        })
        .collect()
}
