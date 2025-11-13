//! Execution of work reports

use super::acc::Accumulated;
use account::{Account, Accounts};
use pvm::{AccumulateState, Pvm};
use score::{
    Gas, ServiceId,
    service::WorkReport,
    vm::{AccumulateItem, DeferredTransfer},
};
use std::collections::{BTreeMap, BTreeSet};

/// (Δ+) outer accumulation
///
/// (N_G, [W], U, D(N_S -> N_G)) -> (N, U, [T], B, U)
///
/// parameters:
/// - N_G: gas limit
/// - [W]: work reports
/// - U: state context
/// - D(N_S -> N_G): gas table
///
/// returns:
/// - N: the number of work-results accumulated.
/// - U: A posterior state-context.
/// - [T]: resultant deferred-transfers
/// - B: accumulation-output pairings.
/// - U: the total gas used
pub async fn outer<V: Pvm, R: Accounts>(
    mut gas_limit: Gas,
    mut transfers: Vec<DeferredTransfer>,
    mut reports: &[WorkReport],
    context: AccumulateState<R>,
    gas_table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated<R> {
    let mut accumulated = Accumulated::new(context);
    loop {
        let mut cumulative_gas = 0;
        let mut index = 0;
        for (i, report) in reports.iter().enumerate() {
            let report_gas: Gas = report.results.iter().map(|r| r.accumulate_gas).sum();
            if cumulative_gas + report_gas > gas_limit {
                break;
            }

            cumulative_gas += report_gas;
            index = i + 1;
        }

        if index == 0 && transfers.is_empty() {
            break;
        }

        let mut step = self::parallel::<V, R>(
            accumulated.context.clone(),
            &transfers,
            if index == 0 { &[] } else { &reports[..index] },
            gas_table,
        )
        .await;

        step.defer_transfers();
        gas_limit -= step.gas.values().sum::<Gas>();
        reports = &reports[index..];
        transfers = step.transfers.clone();
        accumulated.transfers.extend(step.transfers);
        accumulated.accumulated += step.accumulated;
        accumulated.context = step.context;
        accumulated.pairings.extend(step.pairings);
        accumulated.pairings.sort_by_key(|(service, _)| *service);
        for (service, gas) in step.gas.iter() {
            *accumulated.gas.entry(*service).or_insert(0) += gas;
        }
    }

    accumulated
}

/// (Δ*) parallel accumulation
pub async fn parallel<V: Pvm, R: Accounts>(
    mut context: AccumulateState<R>,
    transfers: &[DeferredTransfer],
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated<R> {
    let mut services: BTreeSet<ServiceId> = Default::default();
    for report in reports {
        for result in &report.results {
            services.insert(result.service_id);
        }
    }

    for &service in table.keys() {
        services.insert(service);
    }

    for transfer in transfers.iter() {
        services.insert(transfer.recipient);
    }

    tracing::debug!("services: {:?}", services);

    // NOTE: this is for debugging usage
    let mut results = {
        let mut results = BTreeMap::new();
        for service in services.iter().cloned() {
            let transfers = transfers
                .iter()
                .filter(|t| t.recipient == service)
                .cloned()
                .collect();
            let result = self::once::<V, R>(context.clone(), transfers, reports, table, service);
            results.insert(service, result);
        }
        results
    };

    /* // Execute each service exactly once using Δ₁ (once function)
    let mut results = if services.len() > 1 {
        let mut pool = tokio::task::JoinSet::new();
        for service in services.iter().cloned() {
            let context = context.clone();
            let reports = reports.to_vec();
            let table = table.clone();
            let transfers = transfers
                .iter()
                .filter(|t| t.recipient == service)
                .cloned()
                .collect();
            pool.spawn_blocking(move || {
                let result =
                    self::once::<V, R>(context, transfers, &reports, &table, service, timeslot);
                (service, result)
            });
        }

        let mut results = BTreeMap::new();
        while let Some(Ok((service, result))) = pool.join_next().await {
            results.insert(service, result);
        }
        results
    } else {
        let service = services.iter().next().expect("should not fail");
        let transfers = transfers
            .iter()
            .filter(|t| t.recipient == *service)
            .cloned()
            .collect();
        let result = self::once::<V, R>(
            context.clone(),
            transfers,
            reports,
            table,
            *service,
            timeslot,
        );
        BTreeMap::from([(*service, result)])
    }; */

    // Extract privilege service results from the already-executed results
    if let Some(result) = results.get(&context.privileges.bless) {
        context.privileges = result.context.privileges.clone();
    };

    // update the validators
    if let Some(result) = results.get(&context.privileges.designate) {
        context.validators = result.context.validators;
    };

    // Handle the assign array - each core has its own assign service
    for (core_index, assign_service) in context.privileges.assign.iter().enumerate() {
        if let Some(result) = results.get(assign_service) {
            context.authorization[core_index] = result.context.authorization[core_index].clone();
        }
    }

    // Update the state of accounts
    let mut gas = BTreeMap::new();
    let mut transfers = Vec::new();
    let mut pairings = Vec::new();
    for (service_id, result) in results.iter_mut() {
        if result.gas == 0 {
            continue;
        }

        let accounts = result.context.accounts.accounts();
        for (id, account) in accounts.iter() {
            if (account.creation() == context.timeslot && context.accounts.get(*id).is_none())
                || id == service_id
            {
                context.accounts.upsert(*id, account.clone());
            }
        }

        for service in result.context.accounts.removed() {
            context.accounts.remove(service);
        }

        transfers.extend(result.transfers.clone());
        gas.insert(*service_id, result.gas);
        if let Some(hash) = result.hash {
            pairings.push((*service_id, hash));
            pairings.sort_by_key(|(service, _)| *service);
        }
    }

    Accumulated {
        accumulated: reports.len(),
        context,
        transfers,
        pairings,
        gas,
    }
}

/// (Δ1) single accumulation (12.24)
pub fn once<V: Pvm, R: Accounts>(
    context: AccumulateState<R>,
    transfers: Vec<DeferredTransfer>,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
    service: ServiceId,
) -> pvm::Accumulated<R> {
    let transfers = transfers
        .iter()
        .filter(|t| t.recipient == service)
        .cloned()
        .collect::<Vec<DeferredTransfer>>();

    let gas = *table.get(&service).unwrap_or(&0)
        + reports
            .iter()
            .flat_map(|r| &r.results)
            .filter(|result| result.service_id == service)
            .map(|result| result.accumulate_gas)
            .sum::<Gas>()
        + transfers.iter().map(|t| t.gas_limit).sum::<Gas>();

    let mut items: Vec<AccumulateItem> = transfers.into_iter().map(AccumulateItem::from).collect();
    for report in reports {
        items.extend(
            report
                .operands(service)
                .into_iter()
                .map(AccumulateItem::from),
        );
    }

    V::accumulate(context, service, gas, items)
}
