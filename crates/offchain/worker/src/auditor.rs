//! Auditor utilities for work-report verification

use crate::{Guarantor, WorkPackageBundle, d3l::Justification};
use anyhow::Result;
use pvm::{Pvm, score::Accounts};
use score::{OpaqueHash, service::WorkReport};

/// Auditor utilities for work-report verification
#[allow(async_fn_in_trait)]
pub trait Auditor: Guarantor {
    /// Verify a shard against its justification
    async fn verify_shard(&self, shard: &[u8], justification: &Justification) -> Result<bool> {
        match justification {
            Justification::Hash(hash) => {
                let shard_hash = crypto::blake2b(shard);
                Ok(shard_hash == *hash)
            }
            Justification::HashPair(left, right) => {
                let shard_hash = crypto::blake2b(shard);
                Ok(shard_hash == *left || shard_hash == *right)
            }
            Justification::SegmentShard(segment_data) => Ok(shard == segment_data.as_slice()),
        }
    }

    /// Reconstruct work-package bundle from available shards
    async fn reconstruct_bundle(&self, erasure_root: OpaqueHash) -> Result<Vec<u8>> {
        // Get all shards
        let shards = self.get_shards(&erasure_root).await?.ok_or_else(|| {
            anyhow::anyhow!("Shards not found for erasure root: {:?}", erasure_root)
        })?;

        // Get layout to know bundle vs segment shards
        let (bundle_count, _) = self.get_shard_layout(&erasure_root).await?.ok_or_else(|| {
            anyhow::anyhow!(
                "Shard layout not found for erasure root: {:?}",
                erasure_root
            )
        })?;

        // Extract only bundle shards for work-package reconstruction
        let bundle_shards: Vec<Vec<u8>> = shards.into_iter().take(bundle_count).collect();

        // Use erasure coding to reconstruct
        let partial = crate::d3l::shard::partial_shards(&bundle_shards);
        let reconstructed = erasure::decode_sync(partial)?;

        Ok(reconstructed)
    }

    /// Audit execution: execute work-package and compare with work-report
    async fn audit<A: Accounts, VM: Pvm>(
        &self,
        core_index: u16,
        bundle: &[u8],
        report: &WorkReport,
        accounts: &mut A,
    ) -> Result<bool> {
        let bundle: WorkPackageBundle = codec::decode(bundle)?;
        let result = self
            .compute::<A, VM>(
                core_index,
                bundle.extrinsic.values().cloned().collect(),
                &bundle.package,
                accounts,
            )
            .await?;

        Ok(result.0 == *report)
    }
}

impl<T: Guarantor> Auditor for T {}
