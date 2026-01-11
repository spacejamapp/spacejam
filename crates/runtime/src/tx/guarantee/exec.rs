//! Execution of work reports

use super::acc::Accumulated;
use account::{Account, Accounts};
use pvm::{AccumulateState, Pvm};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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
pub fn outer<V: Pvm, R: Accounts>(
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
        );

        step.defer_transfers();
        gas_limit -= step.gas.values().sum::<Gas>();
        reports = &reports[index..];
        transfers = step.transfers.clone();
        accumulated.transfers.extend(step.transfers);
        accumulated.accumulated += step.accumulated;
        accumulated.context = step.context;
        accumulated.pairings.extend(step.pairings);
        for (service, gas) in step.gas.iter() {
            *accumulated.gas.entry(*service).or_insert(0) += gas;
        }
    }

    accumulated
}

/// (Δ*) parallel accumulation
pub fn parallel<V: Pvm, R: Accounts>(
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

    let mut results = services
        .iter()
        .map(|service| {
            let transfers = transfers
                .par_iter()
                .filter(|t| t.recipient == *service)
                .cloned()
                .collect();
            let result = self::once::<V, R>(context.clone(), transfers, reports, table, *service);
            (*service, result)
        })
        .collect::<BTreeMap<ServiceId, pvm::Accumulated<R>>>();

    // Helper function R(o, a, b) from graypaper: if manager changed it (a != o), use a; else use b
    let r = |old: ServiceId, mgr: ServiceId, svc: ServiceId| -> ServiceId {
        if mgr == old { svc } else { mgr }
    };

    // Get manager service post-state
    let mgr = results
        .get(&context.privileges.bless)
        .map(|r| &r.context.privileges);
    if let Some(mgr) = mgr {
        context.privileges.bless = mgr.bless;
        context.privileges.always_acc = mgr.always_acc.clone();
    }

    // Update assign services
    for (c, old) in context.privileges.assign.into_iter().enumerate() {
        let mgr_val = mgr.map(|m| m.assign[c]).unwrap_or(old);
        let svc_val = results.get(&old).map(|r| r.context.privileges.assign[c]);
        context.privileges.assign[c] = svc_val.map(|s| r(old, mgr_val, s)).unwrap_or(mgr_val);
    }

    // Update designate
    let designate = context.privileges.designate;
    let mgr_designate = mgr
        .map(|m| m.designate)
        .unwrap_or(context.privileges.designate);
    let svc_designate = results
        .get(&designate)
        .map(|r| r.context.privileges.designate);
    context.privileges.designate = svc_designate
        .map(|s| r(designate, mgr_designate, s))
        .unwrap_or(mgr_designate);

    // Update register
    let register = context.privileges.register;
    let mgr_register = mgr.map(|m| m.register).unwrap_or(register);
    let svc_register = results
        .get(&register)
        .map(|r| r.context.privileges.register);
    context.privileges.register = svc_register
        .map(|s| r(register, mgr_register, s))
        .unwrap_or(mgr_register);

    // Update validators from the (now potentially updated) designate service
    // This must happen AFTER privilege updates to read from the correct designate service
    if let Some(result) = results.get(&context.privileges.designate) {
        context.validators = result.context.validators;
    }

    // Update the state of accounts
    let mut gas = BTreeMap::new();
    let mut transfers = Vec::new();
    let mut pairings = BTreeSet::new();
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
            pairings.insert((*service_id, hash));
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
