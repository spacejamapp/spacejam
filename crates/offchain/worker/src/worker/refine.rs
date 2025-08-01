//! refinement

use crate::{SegmentProvider, Worker};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{RefineLoad, WorkExecResult, WorkPackage, WorkReport, WorkResult},
    Accounts, Segment, TimeSlot,
};

impl<S: SegmentProvider> Worker<S> {
    /// Get segment justification for an import specification
    async fn get_segment_justification(
        &self,
        import_spec: &score::service::ImportSpec,
    ) -> crate::segment::Justification {
        // Try to get justification with default shard_index 0
        if let Ok(Some(segment_justification)) = self
            .segment_provider
            .segment_justification(&import_spec.tree_root, import_spec.index, 0)
            .await
        {
            // Use the first justification in the path, or fall back to hash
            segment_justification
                .path
                .path
                .first()
                .cloned()
                .unwrap_or(crate::segment::Justification::Hash(import_spec.tree_root))
        } else {
            // Fallback: use the tree_root as a hash justification
            crate::segment::Justification::Hash(import_spec.tree_root)
        }
    }

    /// Process all work items with Refine invocations using segment provider
    pub async fn refine<R: Accounts, VM: Pvm>(
        &mut self,
        work: &WorkPackage,
        accounts: &mut R,
        core_idx: u16,
        report: &mut WorkReport,
    ) -> Result<()> {
        // Import segments, collect extrinsics and justifications
        let mut all_imports = Vec::new();
        let mut all_import_segments = Vec::new();
        let mut all_extrinsics = Vec::new();
        let mut justifications = Vec::new();
        for item in &work.items {
            // Import segments
            let segments = self.import_segments(item).await?;
            all_import_segments.extend_from_slice(&segments);
            all_imports.push(segments);

            // Collect extrinsics
            for ext_spec in &item.extrinsic {
                let ext_data = self.extrinsic_data.get(&ext_spec.hash).ok_or_else(|| {
                    anyhow::anyhow!("Missing extrinsic data for hash {:?}", ext_spec.hash)
                })?;

                if ext_data.len() != ext_spec.len as usize {
                    return Err(anyhow::anyhow!(
                        "Extrinsic data length mismatch for hash {:?}: expected {}, got {}",
                        ext_spec.hash,
                        ext_spec.len,
                        ext_data.len()
                    ));
                }
                all_extrinsics.push(ext_data.clone());
            }

            // Collect justifications for imported segments
            for import_spec in &item.import_segments {
                justifications.push(self.get_segment_justification(import_spec).await);
            }
        }

        // Execute refinement on all work items
        //
        // TODO: Process work items in parallel if possible
        let mut work_results = Vec::new();
        let mut total_exports = 0u16;
        let mut all_exported_segments = Vec::new();
        for (item_index, item) in work.items.iter().enumerate() {
            let (work_result, exported) = self
                .refine_single::<R, VM>(
                    core_idx,
                    work.context.lookup_anchor_slot,
                    work,
                    item_index,
                    accounts,
                    total_exports,
                    &all_imports,
                    &report.auth_output,
                )
                .await?;

            total_exports += item.export_count;
            all_exported_segments.extend(exported);
            work_results.push(work_result);
        }

        // Export all segments together if any were produced
        if !all_exported_segments.is_empty() {
            let package_hash = crypto::blake2b(&codec::encode(work)?);
            self.export_segments(&all_exported_segments, &package_hash)
                .await?;
        }

        // Compute erasure root from all segments
        let bundle = crate::SegmentBundle {
            package: work.clone(),
            extrinsics: all_extrinsics,
            imports: all_import_segments,
            justifications,
        };
        let erasure_root = bundle.erasure_root(&all_exported_segments)?;

        // Compute exports root from exported segments
        let exports_root = if all_exported_segments.is_empty() {
            [0u8; 32]
        } else {
            crypto::merkle::hroot(
                &all_exported_segments
                    .iter()
                    .map(|seg| crypto::blake2b(seg))
                    .collect::<Vec<_>>(),
            )
        };

        // Update work report with final results
        report.spec.exports_count = work_results.iter().map(|r| r.refine_load.exports).sum();
        report.spec.exports_root = exports_root;
        report.spec.erasure_root = erasure_root;
        report.results = work_results;
        Ok(())
    }

    /// Process a single work item with segment provider
    #[allow(clippy::too_many_arguments)]
    async fn refine_single<R: Accounts, VM: Pvm>(
        &self,
        core: u16,
        timeslot: TimeSlot,
        package: &WorkPackage,
        item_index: usize,
        accounts: &mut R,
        export_offset: u16,
        all_imports: &[Vec<Segment>],
        auth_output: &[u8],
    ) -> Result<(WorkResult, Vec<Segment>)> {
        let item = &package.items[item_index];

        // Execute Refine invocation (Ψ_R)
        let refined = VM::refine(
            core,
            item_index,
            package,
            auth_output,
            all_imports,
            export_offset,
            accounts,
            timeslot,
        );

        // Validate output size and construct result
        let result = match refined.executed.exec {
            WorkExecResult::Ok(output) if output.len() <= score::MAX_WORK_REPORT_OUTPUT_SIZE => {
                WorkExecResult::Ok(output)
            }
            WorkExecResult::Ok(_) => {
                return Err(anyhow::anyhow!("Work item output size exceeded"));
            }
            other => other,
        };

        // Create work result with load metrics
        let work_result = WorkResult {
            service_id: item.service,
            code_hash: item.code_hash,
            payload_hash: crypto::blake2b(&item.payload),
            accumulate_gas: item.accumulate_gas_limit,
            result,
            refine_load: RefineLoad {
                gas_used: refined.executed.gas,
                imports: item.import_segments.len() as u16,
                extrinsic_count: item.extrinsic.len() as u16,
                extrinsic_size: item.extrinsic.iter().map(|e| e.len).sum(),
                exports: refined.segments.len() as u16,
            },
        };

        Ok((work_result, refined.segments))
    }
}
