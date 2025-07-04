//! refinement

use crate::{worker::Worker, Config};
use anyhow::Result;
use pvm::Invocation;
use score::{
    service::{RefineLoad, WorkExecResult, WorkItem, WorkPackage, WorkResult},
    Account, Accounts,
};

impl<'a, C: Config> Worker<'a, C> {
    /// Phase 3: Process all work items with Refine invocations
    pub fn refine<R: Accounts>(&mut self, work: &WorkPackage, accounts: &mut R) -> Result<()> {
        let mut work_results = Vec::new();
        let mut total_exports = 0u16;

        // TODO: doing this in parallel if possible
        for (item_index, item) in work.items.iter().enumerate() {
            let work_result =
                self.refine_single(item, item_index, &work.context, accounts, total_exports)?;

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
        item: &WorkItem,
        item_index: usize,
        context: &score::service::RefineContext,
        accounts: &mut R,
        export_offset: u16,
    ) -> Result<WorkResult> {
        // Get service account for this work item
        let Some(service_account) = accounts.get(item.service) else {
            anyhow::bail!(
                "Service {} not found for work item {}",
                item.service,
                item_index
            );
        };

        // Historical lookup for service code
        let service_code = service_account
            .historical_lookup(context.lookup_anchor_slot, item.code_hash)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Service code with hash {:?} not available at timeslot {}",
                    item.code_hash,
                    context.lookup_anchor_slot
                )
            })?;

        // Import segments for this work item
        let _extrinsic_data = self.extrinsic(item, accounts)?;

        // Execute Refine invocation (Ψ_R)
        //
        // TODO: implement the refine call.
        let refine_result = C::Vm::refine(item_index, item, &service_code, &[], export_offset);

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

    /// Get extrinsic data for a work item
    fn extrinsic<R: score::Accounts>(
        &self,
        item: &WorkItem,
        _accounts: &R,
    ) -> Result<Vec<Vec<u8>>> {
        let mut extrinsic_data = Vec::new();

        for extrinsic_spec in &item.extrinsic {
            // TODO: Implement extrinsic data retrieval
            // This would involve looking up the preimage of the extrinsic hash
            extrinsic_data.push(vec![0u8; extrinsic_spec.len as usize]);
        }

        Ok(extrinsic_data)
    }
}
