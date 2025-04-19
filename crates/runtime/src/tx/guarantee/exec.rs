//! Execution of work reports

use pvm::{AccumulateResult, Pvm};
use score::{
    Gas, ServiceId,
    service::WorkReport,
    vm::{Accumulated, StateContext},
};
use std::collections::BTreeMap;

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
    context: StateContext,
    gas_table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated {
    // NOTE: the graypaper is using max(N|w| + 1), I believe it is a typo.
    let count = reports
        .iter()
        .filter(|r| r.results.iter().map(|r| r.accumulate_gas).sum::<Gas>() <= gas_limit)
        .count();
    if count == 0 {
        return Accumulated {
            context,
            ..Default::default()
        };
    }

    let mut accumulated = self::parallel::<V>(context.clone(), &reports[..count], gas_table);
    let rest = self::outer::<V>(
        gas_limit - accumulated.gas.values().sum::<Gas>(),
        &reports[count..],
        accumulated.context.clone(),
        &Default::default(),
    );

    accumulated.accumulated += rest.accumulated;
    accumulated.gas.extend(rest.gas);
    accumulated.transfers.extend(rest.transfers);
    accumulated.pairings.extend(rest.pairings);
    accumulated
}

/// (Δ*) parallel accumulation
pub fn parallel<V: Pvm>(
    context: StateContext,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated {
    // TODO: use a local task pool for spawning this calculation.
    let services = table.keys().collect::<Vec<_>>();
    tracing::trace!("parallel accumulation for services {:?}", services);
    let results = services
        .iter()
        .map(|service| {
            (
                **service,
                self::once::<V>(context.clone(), reports, table, **service),
            )
        })
        .collect::<Vec<_>>();

    // assemble the result
    let mut accounts = context.accounts.clone();
    let mut gas = BTreeMap::new();
    let mut removed: Vec<ServiceId> = vec![];
    let mut transfers = vec![];
    let mut pairings = BTreeMap::new();
    for (service_id, result) in results.into_iter() {
        // new accounts
        for (id, account) in result.context.accounts.iter() {
            if !services.contains(&id) {
                accounts.insert(*id, account.clone());
            }
        }

        // removed accounts
        //
        // TODO: find a better way to do this.
        for service in result.context.accounts.keys() {
            if !services.contains(&service) {
                removed.push(*service);
            }
        }

        // update other fields
        gas.insert(service_id, result.gas);
        transfers.extend(result.transfers);
        if let Some(hash) = result.hash {
            pairings.insert(service_id, hash);
        }
    }

    // remove the removed accounts
    for service in removed {
        accounts.remove(&service);
    }

    // get next context
    //
    // TODO: use a local task pool for spawning this calculation.
    let results = [
        context.privileges.bless,
        context.privileges.designate,
        context.privileges.assign,
    ]
    .iter()
    .map(|service| self::once::<V>(context.clone(), reports, table, *service))
    .collect::<Vec<_>>();

    let (privileges, validators, authorization) = (
        results[0].context.privileges.clone(),
        results[1].context.validators.clone(),
        results[2].context.authorization.clone(),
    );

    Accumulated {
        accumulated: reports.len(),
        context: StateContext {
            accounts,
            privileges,
            validators,
            authorization,
        },
        transfers,
        pairings,
        gas,
    }
}

/// (Δ1) single accumulation
pub fn once<V: Pvm>(
    context: StateContext,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
    service: ServiceId,
) -> AccumulateResult {
    let gas = *table.get(&service).unwrap_or(&0)
        + reports
            .iter()
            .map(|r| r.results.iter().map(|r| r.accumulate_gas).sum::<Gas>())
            .sum::<Gas>();

    let operands = reports
        .iter()
        .flat_map(|r| r.operands(service))
        .collect::<Vec<_>>();

    V::accumulate(context, 0, service, gas, operands, [0; 32])
}

/*    /// integrate the deferred transfers
pub fn integrate<V: Pvm>(
    accounts: &mut BTreeMap<ServiceId, ServiceAccount>,
    transfers: &[DeferredTransfer],
    slot: TimeSlot,
) -> anyhow::Result<Gas> {
    let mut gas_used = 0;
    // Process each account in the intermediate state
    for (service_id, _account) in accounts.clone().into_iter() {
        let transfers = DeferredTransfer::select(transfers, service_id);
        if transfers.is_empty() {
            continue;
        }

        // Invoke PVM's transfer function (Ψ_T) for this service
        // This applies all transfers targeting this service in order
        //
        // TODO: handle the changes of accounts may be using smart pointer.
        let (new_account, gas) = V::transfer(accounts, slot, service_id, &transfers);

        gas_used += gas;
        accounts.insert(service_id, new_account);
    }

    Ok(gas_used)
} */
