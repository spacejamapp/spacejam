//! Execution of work reports

use pvm::Pvm;
use score::{
    service::WorkReport,
    vm::{AccumulateState, Accumulated},
    Account, Accounts, Gas, ServiceId, TimeSlot,
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
    mut reports: &[WorkReport],
    context: AccumulateState<R>,
    gas_table: &BTreeMap<ServiceId, Gas>,
    timeslot: TimeSlot,
) -> Accumulated<R> {
    let mut accumulated = Accumulated::new(context);
    loop {
        let mut cumulative_gas = 0;
        let mut index = 0;
        for (i, report) in reports.iter().enumerate() {
            let report_gas: Gas = report.results.iter().map(|r| r.accumulate_gas).sum();
            if cumulative_gas + report_gas <= gas_limit {
                cumulative_gas += report_gas;
                index = i + 1;
            }
        }

        if index == 0 {
            return accumulated;
        }

        let step = self::parallel::<V, R>(
            accumulated.context.clone(),
            &reports[..index],
            gas_table,
            timeslot,
        )
        .await;

        gas_limit -= step.gas.values().sum::<Gas>();
        reports = &reports[index..];
        accumulated.accumulated += step.accumulated;
        accumulated.transfers.extend(step.transfers);
        accumulated.pairings.extend(step.pairings);
        accumulated.context = step.context;
        for (service, gas) in step.gas.iter() {
            *accumulated.gas.entry(*service).or_insert(0) += gas;
        }
    }
}

/// (Δ*) parallel accumulation
pub async fn parallel<V: Pvm, R: Accounts>(
    mut context: AccumulateState<R>,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
    timeslot: TimeSlot,
) -> Accumulated<R> {
    let mut services: BTreeSet<ServiceId> = table.keys().cloned().collect();
    for report in reports {
        for result in &report.results {
            services.insert(result.service_id);
        }
    }

    // Execute each service exactly once using Δ₁ (once function)
    let mut results = if services.len() > 1 {
        let mut pool = tokio::task::JoinSet::new();
        for service in services.iter().cloned() {
            let context = context.clone();
            let reports = reports.to_vec();
            let table = table.clone();
            pool.spawn_blocking(move || {
                let result = self::once::<V, R>(context, &reports, &table, service, timeslot);
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
        let result = self::once::<V, R>(context.clone(), &reports, &table, *service, timeslot);
        BTreeMap::from([(*service, result)])
    };

    // Update the state of accounts
    let mut removed = BTreeSet::new();
    let mut gas = BTreeMap::new();
    let mut transfers = vec![];
    let mut pairings = BTreeMap::new();
    let services = context.accounts.services();
    for (service_id, result) in results.iter_mut() {
        let lsvc = result.context.accounts.services();
        let accounts = result.context.accounts.accounts();
        for (id, account) in accounts.iter() {
            if account.creation() == timeslot || id == service_id {
                let mut account = account.clone();
                if id == service_id {
                    account.set_update(timeslot);
                }

                context.accounts.upsert(*id, account);
            }
        }

        for removed in result.context.accounts.removed() {
            context.accounts.remove(removed);
        }

        for account_id in &services {
            if !lsvc.contains(account_id) {
                removed.insert(account_id);
            }
        }

        transfers.extend(result.transfers.clone());
        gas.insert(*service_id, result.gas);
        if let Some(hash) = result.hash {
            pairings.insert(*service_id, hash);
        }
    }

    // Remove accounts that were removed by any service
    for account_id in removed {
        context.accounts.remove(*account_id);
    }

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

    Accumulated {
        accumulated: reports.len(),
        context,
        transfers,
        pairings,
        gas,
    }
}

/// (Δ1) single accumulation
pub fn once<V: Pvm, R: Accounts>(
    context: AccumulateState<R>,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
    service: ServiceId,
    timeslot: TimeSlot,
) -> pvm::Accumulated<R> {
    let gas = *table.get(&service).unwrap_or(&0)
        + reports
            .iter()
            .flat_map(|r| &r.results)
            .filter(|result| result.service_id == service)
            .map(|result| result.accumulate_gas)
            .sum::<Gas>();

    let operands = reports
        .iter()
        .flat_map(|r| r.operands(service))
        .collect::<Vec<_>>();
    V::accumulate(context, timeslot, service, gas, operands)
}
