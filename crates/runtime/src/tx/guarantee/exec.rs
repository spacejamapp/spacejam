//! Execution of work reports

use crate::Storage;
use pvm::Pvm;
use score::{
    Gas, ServiceId,
    service::WorkReport,
    vm::{AccumulateResult, Accumulated, StateContext},
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
    _gas_limit: Gas,
    _reports: Vec<WorkReport>,
    context: StateContext,
    _accounts: &impl Storage,
    _gas_table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated {
    let _ = V::accumulate(context, 0, 0, 0, Default::default(), [0; 32]);
    Default::default()
}

/// (Δ*) parallel accumulation
pub fn parallel<V: Pvm>(
    context: StateContext,
    reports: Vec<WorkReport>,
    table: BTreeMap<ServiceId, Gas>,
) -> Accumulated {
    // TODO: use a local task pool for spawning this calculation.
    let services = table.keys().collect::<Vec<_>>();
    let results = services
        .iter()
        .map(|service| {
            (
                **service,
                self::once::<V>(context.clone(), reports.clone(), &table, **service),
            )
        })
        .collect::<Vec<_>>();

    // assemble the result
    let mut accounts = context.accounts.clone();
    let mut gas = 0;
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
        gas += result.gas;
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
    .map(|service| self::once::<V>(context.clone(), reports.clone(), &table, *service))
    .collect::<Vec<_>>();

    let (privileges, validators, authorization) = (
        results[0].context.privileges.clone(),
        results[1].context.validators.clone(),
        results[2].context.authorization.clone(),
    );

    Accumulated {
        accumulated: 0,
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
    reports: Vec<WorkReport>,
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
