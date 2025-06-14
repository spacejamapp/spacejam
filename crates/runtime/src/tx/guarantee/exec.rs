//! Execution of work reports

use pvm::{Accounts, AccumulateResult, Pvm};
use score::{
    Gas, ServiceId, TimeSlot,
    service::WorkReport,
    vm::{Accumulated, StateContext},
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
pub fn outer<V: Pvm>(
    gas_limit: Gas,
    reports: &[WorkReport],
    context: StateContext<V::Accounts>,
    gas_table: &BTreeMap<ServiceId, Gas>,
    timeslot: TimeSlot,
) -> Accumulated<V::Accounts> {
    // NOTE: we might need to sort the reports by the gas limit,
    // need to double check if we have already done it.
    //
    // do we need to check the gas used in always accumulate as well?
    let mut cumulative_gas = 0;
    let index = reports
        .iter()
        .take_while(|r| {
            let report_gas: Gas = r.results.iter().map(|r| r.accumulate_gas).sum();
            cumulative_gas += report_gas;
            cumulative_gas <= gas_limit
        })
        .count();

    if index == 0 {
        return Accumulated::new(context);
    }

    let mut accumulated =
        self::parallel::<V>(context.clone(), &reports[..index], gas_table, timeslot);

    // TODO: re-check if we need a loop here.
    let rest = self::outer::<V>(
        gas_limit - accumulated.gas.values().sum::<Gas>(),
        &reports[index..],
        accumulated.context.clone(),
        gas_table,
        timeslot,
    );

    accumulated.accumulated += rest.accumulated;
    accumulated.gas.extend(rest.gas);
    accumulated.transfers.extend(rest.transfers);
    accumulated.pairings.extend(rest.pairings);
    accumulated
}

/// (Δ*) parallel accumulation
pub fn parallel<V: Pvm>(
    mut context: StateContext<V::Accounts>,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
    timeslot: TimeSlot,
) -> Accumulated<V::Accounts> {
    // FIXME: extract the services from reports
    let mut services: BTreeSet<ServiceId> = table.keys().cloned().collect();
    for report in reports {
        for result in &report.results {
            services.insert(result.service_id);
        }
    }

    // Execute each service exactly once using Δ₁ (once function)
    let results: BTreeMap<ServiceId, AccumulateResult<V::Accounts>> = services
        .iter()
        .map(|service| {
            let result = self::once::<V>(context.clone(), reports, table, *service, timeslot);
            (*service, result)
        })
        .collect();

    // Update the state of accounts
    let mut removed = BTreeSet::new(); // m
    let mut gas = BTreeMap::new();
    let mut transfers = vec![];
    let mut pairings = BTreeMap::new();
    let services = context.accounts.services();
    for (service_id, result) in results.iter() {
        let lsvc = result.context.accounts.services();
        let accounts = result.context.accounts.clone().accounts();
        for (id, account) in accounts.into_iter() {
            // FIXME:
            //
            // - check if we do need update the accounts
            // - handle the same code different services logic more carefully
            if !services.contains(&id) || id == *service_id {
                context.accounts.upsert(id, account.clone());
            }
        }

        for account_id in &services {
            if !lsvc.contains(account_id) {
                removed.insert(account_id);
            }
        }

        // Collect other outputs
        gas.insert(*service_id, result.gas);
        transfers.extend(result.transfers.clone());
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

    if let Some(result) = results.get(&context.privileges.designate) {
        context.validators = result.context.validators.clone();
    };

    if let Some(result) = results.get(&context.privileges.assign) {
        context.authorization = result.context.authorization.clone();
    };

    Accumulated {
        accumulated: reports.len(),
        context,
        transfers,
        pairings,
        gas,
    }
}

/// (Δ1) single accumulation
pub fn once<V: Pvm>(
    context: StateContext<V::Accounts>,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
    service: ServiceId,
    timeslot: TimeSlot,
) -> AccumulateResult<V::Accounts> {
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
    V::accumulate(context, timeslot, service, gas, operands, [0; 32])
}
