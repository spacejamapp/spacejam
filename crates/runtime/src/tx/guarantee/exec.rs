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
    validators: &mut score::safrole::ValidatorsData,
    gas_table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated<R> {
    let mut accumulated = Accumulated::new(context);
    let empty = BTreeMap::new();
    let mut first = true;
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

        if index == 0 && transfers.is_empty() && (!first || gas_table.is_empty()) {
            break;
        }

        let step = self::parallel::<V, R>(
            accumulated.context.clone(),
            validators,
            &transfers,
            if index == 0 { &[] } else { &reports[..index] },
            if first { gas_table } else { &empty },
        );

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
        first = false;

        if reports.is_empty() && transfers.is_empty() {
            break;
        }
    }

    accumulated
}

/// (Δ*) parallel accumulation
pub fn parallel<V: Pvm, R: Accounts>(
    mut context: AccumulateState<R>,
    validators: &mut score::safrole::ValidatorsData,
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

    let designate = context.privileges.designate;
    let validators_ref = &*validators;
    let mut results = services
        .par_iter()
        .map(|service| {
            let v = if *service == designate {
                validators_ref.clone()
            } else {
                Default::default()
            };
            let transfers = transfers
                .par_iter()
                .filter(|t| t.recipient == *service)
                .cloned()
                .collect();
            let result =
                self::once::<V, R>(context.clone(), v, transfers, reports, table, *service);
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
    for c in 0..context.privileges.assign.len() {
        let old = context.privileges.assign[c];
        let mgr_val = mgr.map(|m| m.assign[c]).unwrap_or(old);
        let svc_val = results.get(&old).map(|r| r.context.privileges.assign[c]);
        context.privileges.assign[c] = svc_val.map(|s| r(old, mgr_val, s)).unwrap_or(mgr_val);
    }

    // Update designate
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

    // Update validators in-place from the designate service (ι')
    if let Some(result) = results.get(&designate) {
        *validators = result.validators.clone();
    }

    // Per graypaper accpar: accounts' = (accounts ∪ n) \ m
    // Collect all removals (m) first so they take precedence over additions.
    let mut removed = BTreeSet::new();
    let mut gas = BTreeMap::new();
    let mut transfers = Vec::new();
    let mut pairings = BTreeSet::new();
    for (service_id, result) in results.iter_mut() {
        transfers.extend(result.transfers.clone());

        if let Some(hash) = result.hash {
            pairings.insert((*service_id, hash));
        }

        if result.gas == 0 {
            continue;
        }

        for service in result.context.accounts.removed() {
            removed.insert(service);
        }

        gas.insert(*service_id, result.gas);
    }

    // Apply additions (n): the service's own account from every result,
    for (service_id, result) in results.iter() {
        if !removed.contains(service_id)
            && let Some(account) = result.context.accounts.accounts().get(service_id)
        {
            context.accounts.upsert(*service_id, account.clone());
        }

        if result.gas == 0 {
            continue;
        }

        for (id, account) in result.context.accounts.accounts().iter() {
            if id == service_id {
                continue; // already handled above
            }
            if !removed.contains(id)
                && account.creation() == context.timeslot
                && context.accounts.get(*id).is_none()
            {
                context.accounts.upsert(*id, account.clone());
            }
        }
    }

    // Apply removals (m)
    for service in &removed {
        context.accounts.remove(*service);
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
    validators: score::safrole::ValidatorsData,
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

    V::accumulate(context, validators, service, gas, items)
}
