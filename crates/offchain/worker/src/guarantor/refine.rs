//! Refine interface

use std::collections::BTreeMap;

use crate::{bundle::WorkPackageBundle, Guarantor};
use anyhow::Result;
use pvm::Pvm;
use score::{
    service::{RefineLoad, WorkExecResult, WorkPackage, WorkReport, WorkResult},
    Accounts, CoreIndex, OpaqueHash, Segment, TimeSlot,
};

/// Refine the work package
pub async fn refine<R: Accounts, VM: Pvm>(
    d3l: &impl Guarantor,
    work: &WorkPackage,
    extrinsic: &BTreeMap<OpaqueHash, Vec<u8>>,
    core_idx: u16,
    auth_output: &[u8],
    accounts: &mut R,
) -> Result<(WorkReport, WorkPackageBundle)> {
    let mut all_imports = Vec::new();
    let mut all_justifications = Vec::new();
    let mut imports_with_proofs = Vec::new();
    for item in &work.items {
        // Import segments
        let segments = d3l
            .import_segments(
                &item
                    .import_segments
                    .iter()
                    .map(|s| s.tree_root)
                    .collect::<Vec<_>>(),
            )
            .await?;

        // Collect justifications for imported segments
        let mut proofs = Vec::new();
        for import_spec in &item.import_segments {
            let justification = if let Some(justification) = d3l
                .segment_justification(&import_spec.tree_root, import_spec.index, 0)
                .await?
            {
                justification
                    .path
                    .path
                    .first()
                    .cloned()
                    .unwrap_or(crate::d3l::Justification::Hash(import_spec.tree_root))
            } else {
                crate::d3l::Justification::Hash(import_spec.tree_root)
            };
            all_justifications.push(justification.clone());
            proofs.push(justification);
        }

        // Collect imports with proofs
        imports_with_proofs.push((
            segments.iter().flatten().copied().collect::<Vec<_>>(),
            proofs,
        ));

        all_imports.push(segments);
    }

    // Execute refinement on all work items
    //
    // TODO: Process work items in parallel if possible
    let mut work_results = Vec::new();
    let mut total_exports = 0u16;
    let mut all_exported_segments = Vec::new();
    for (item_index, item) in work.items.iter().enumerate() {
        // TODO: add extrinsic data
        let (work_result, exported) = refine_single::<R, VM>(
            core_idx,
            work.context.lookup_anchor_slot,
            work,
            item_index,
            accounts,
            total_exports,
            &all_imports,
            auth_output,
        )
        .await?;

        total_exports += item.export_count;
        all_exported_segments.extend(exported);
        work_results.push(work_result);
    }

    // Export all segments and get efficient data for specifier creation
    //
    // TODO: cache proofs
    let bundle = WorkPackageBundle {
        package: work.clone(),
        extrinsic: extrinsic.clone(),
        imports_with_proofs,
    };
    let spec = bundle.specify(all_exported_segments).await?;

    Ok((
        WorkReport {
            spec,
            results: work_results,
            context: work.context.clone(),
            core_index: core_idx as CoreIndex,
            authorizer_hash: work.authorizer.hash(),
            auth_output: auth_output.to_vec(),
            lookup: BTreeMap::new(),
            auth_gas_used: 0,
        },
        bundle,
    ))
}

/// Process a single work item with segment provider
#[allow(clippy::too_many_arguments)]
pub async fn refine_single<R: Accounts, VM: Pvm>(
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
