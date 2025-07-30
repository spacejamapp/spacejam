//! refinement

use crate::{SegmentProvider, Worker};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{RefineLoad, WorkExecResult, WorkPackage, WorkResult},
    Accounts, Segment, TimeSlot,
};

impl<P: SegmentProvider> Worker<P> {
    /// Process all work items with Refine invocations using segment provider
    pub async fn refine_with_provider<R: Accounts, VM: Pvm>(
        &mut self,
        work: &WorkPackage,
        accounts: &mut R,
        core_idx: u16,
    ) -> Result<()> {
        // Import all required segments for all work items
        let mut all_imports = Vec::new();
        let mut all_import_segments = Vec::new();

        for item in &work.items {
            let segments = self.import_segments_with_provider(item).await?;
            all_import_segments.extend_from_slice(&segments);
            all_imports.push(segments);
        }

        let mut work_results = Vec::new();
        let mut total_exports = 0u16;
        let mut all_exported_segments = Vec::new();

        // TODO: doing this in parallel if possible
        for (item_index, item) in work.items.iter().enumerate() {
            let (work_result, exported) = self
                .refine_single_with_provider::<R, VM>(
                    core_idx,
                    work.context.lookup_anchor_slot,
                    work,
                    item_index,
                    accounts,
                    total_exports,
                    &all_imports,
                )
                .await?;

            total_exports += item.export_count;
            all_exported_segments.extend(exported);
            work_results.push(work_result);
        }

        // Collect all extrinsic data
        let mut all_extrinsics = Vec::new();
        for item in &work.items {
            for ext_spec in &item.extrinsic {
                // TODO: Fetch actual extrinsic data by hash
                // For now, use placeholder
                all_extrinsics.push(vec![0u8; ext_spec.len as usize]);
            }
        }

        // Create bundle for erasure root computation
        let bundle = crate::SegmentBundle {
            package: work.clone(),
            extrinsics: all_extrinsics,
            imports: all_import_segments,
            justifications: vec![], // TODO: Implement justification collection
        };

        // Compute erasure root according to Gray Paper
        let erasure_root = bundle.erasure_root(&all_exported_segments)?;

        // Compute exports root (merkle root of exported segments)
        let exports_root = if all_exported_segments.is_empty() {
            [0u8; 32]
        } else {
            // Simple merkle root of segment hashes
            let hashes: Vec<_> = all_exported_segments
                .iter()
                .map(|seg| crypto::blake2b(seg))
                .collect();
            crypto::merkle::hroot(&hashes)
        };

        // Update work report
        self.report.spec.exports_count = work_results.iter().map(|r| r.refine_load.exports).sum();
        self.report.spec.exports_root = exports_root;
        self.report.spec.erasure_root = erasure_root;
        self.report.results = work_results;
        Ok(())
    }

    /// Legacy method for backward compatibility
    pub fn refine<R: Accounts, VM: Pvm>(
        &mut self,
        work: &WorkPackage,
        accounts: &mut R,
        core_idx: u16,
    ) -> Result<()> {
        let mut work_results = Vec::new();
        let mut total_exports = 0u16;

        // TODO: doing this in parallel if possible
        for (item_index, item) in work.items.iter().enumerate() {
            let work_result = self.refine_single::<R, VM>(
                core_idx,
                work.context.lookup_anchor_slot,
                work,
                item_index,
                accounts,
                total_exports,
            )?;

            total_exports += item.export_count;
            work_results.push(work_result);
        }

        // update work report
        self.report.spec.exports_count = work_results.iter().map(|r| r.refine_load.exports).sum();
        self.report.spec.exports_root = [0u8; 32]; // TODO: Compute proper exports root from exported segments
        self.report.spec.erasure_root = [0u8; 32]; // TODO: Legacy method doesn't compute proper erasure root
        self.report.results = work_results;
        Ok(())
    }

    /// Process a single work item with segment provider
    #[allow(clippy::too_many_arguments)]
    async fn refine_single_with_provider<R: Accounts, VM: Pvm>(
        &self,
        core: u16,
        timeslot: TimeSlot,
        package: &WorkPackage,
        item_index: usize,
        accounts: &mut R,
        export_offset: u16,
        all_imports: &[Vec<Segment>],
    ) -> Result<(WorkResult, Vec<Segment>)> {
        // Execute Refine invocation (Ψ_R) with imported segments
        let refined = VM::refine(
            core,
            item_index,
            package,
            &self.report.auth_output,
            all_imports,
            export_offset,
            accounts,
            timeslot,
        );

        // Handle segment exports if any were produced
        if !refined.segments.is_empty() {
            let encoded = codec::encode(package)?;
            let package_hash = crypto::blake2b(&encoded);
            let exports_root = self
                .export_segments_with_provider(&refined.segments, &package_hash, &self.provider)
                .await?;

            // Update the exports root in the worker (will be used later)
            // TODO: Store this exports_root properly for the final work report
            tracing::debug!("Exported segments with root: {:?}", exports_root);
        }

        // Check output size constraints and create work result
        let result = match refined.executed.exec {
            WorkExecResult::Ok(output) if output.len() <= score::MAX_WORK_REPORT_OUTPUT_SIZE => {
                WorkExecResult::Ok(output)
            }
            WorkExecResult::Ok(_) => {
                return Err(anyhow::anyhow!("Work item output size exceeded"));
            }
            other => other,
        };

        // Create work result
        let item = &package.items[item_index];
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

    /// Process a single work item (legacy)
    fn refine_single<R: Accounts, VM: Pvm>(
        &self,
        core: u16,
        timeslot: TimeSlot,
        package: &WorkPackage,
        item_index: usize,
        accounts: &mut R,
        export_offset: u16,
    ) -> Result<WorkResult> {
        // Execute Refine invocation (Ψ_R)
        let refined = VM::refine(
            core,
            item_index,
            package,
            &self.report.auth_output,
            &[], // TODO: Pass actual import segments when available
            export_offset,
            accounts,
            timeslot,
        );

        // Check output size constraints and create work result
        let result = match refined.executed.exec {
            WorkExecResult::Ok(output) if output.len() <= score::MAX_WORK_REPORT_OUTPUT_SIZE => {
                WorkExecResult::Ok(output)
            }
            WorkExecResult::Ok(_) => {
                return Err(anyhow::anyhow!("Work item output size exceeded"));
            }
            other => other,
        };

        // Create work result
        let item = &package.items[item_index];
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
                exports: item.export_count,
            },
        };

        // TODO: Handle segment exports with erasure coding
        // This would involve using the erasure coding library to encode exported segments
        // and distribute them according to the availability specifier

        Ok(work_result)
    }
}
