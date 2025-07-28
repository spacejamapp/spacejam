//! refinement

use crate::Worker;
use anyhow::Result;
use pvm::Invocation;
use runtime::Config;
use score::{
    service::{RefineLoad, WorkExecResult, WorkPackage, WorkResult},
    Accounts, TimeSlot,
};

impl<C: Config> Worker<C> {
    /// Process all work items with Refine invocations
    pub fn refine<R: Accounts>(&mut self, work: &WorkPackage, accounts: &mut R, core_idx: u16) -> Result<()> {
        let mut work_results = Vec::new();
        let mut total_exports = 0u16;

        // TODO: doing this in parallel if possible
        for (item_index, item) in work.items.iter().enumerate() {
            let work_result = self.refine_single(
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
        self.report.results = work_results;
        Ok(())
    }

    /// Process a single work item
    fn refine_single<R: score::Accounts>(
        &self,
        core: u16,
        timeslot: TimeSlot,
        package: &WorkPackage,
        item_index: usize,
        accounts: &mut R,
        export_offset: u16,
    ) -> Result<WorkResult> {
        // Execute Refine invocation (Ψ_R)
        let refine_result = C::Vm::refine(
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
        let result = match refine_result.executed.exec {
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
                gas_used: refine_result.executed.gas,
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
