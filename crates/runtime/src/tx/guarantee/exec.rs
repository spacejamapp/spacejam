//! Execution of work reports

use pvm::{AccumulateResult, Pvm};
use score::{
    Gas, ServiceId,
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
    context: StateContext,
    gas_table: &BTreeMap<ServiceId, Gas>,
) -> Accumulated {
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
        return Accumulated {
            context,
            ..Default::default()
        };
    }

    let mut accumulated = self::parallel::<V>(context.clone(), &reports[..index], gas_table);
    let rest = self::outer::<V>(
        gas_limit - accumulated.gas.values().sum::<Gas>(),
        &reports[index..],
        accumulated.context.clone(),
        gas_table,
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
    let mut services: BTreeSet<ServiceId> = table.keys().cloned().collect();
    for report in reports {
        for result in &report.results {
            services.insert(result.service_id);
        }
    }

    let results = services
        .iter()
        .map(|service| {
            let result = self::once::<V>(context.clone(), reports, table, *service);
            (*service, result)
        })
        .collect::<Vec<_>>();

    // assemble the result
    let mut accounts = context.accounts.clone();
    let mut gas = BTreeMap::new();
    let mut removed: Vec<ServiceId> = vec![];
    let mut transfers = vec![];
    let mut pairings = BTreeMap::new();

    for (service_id, result) in results.into_iter() {
        tracing::debug!(
            "Processing service {} result, accounts in result: {:?}",
            service_id,
            result.context.accounts.keys().collect::<Vec<_>>()
        );
        // Update all accounts from the result, not just new ones
        for (id, account) in result.context.accounts.iter() {
            tracing::debug!(
                "Updating account {} with storage entries: {}",
                id,
                account.storage.len()
            );
            accounts.insert(*id, account.clone());
        }

        // removed accounts
        //
        // TODO: find a better way to do this.
        for service in result.context.accounts.keys() {
            if !services.contains(service) {
                tracing::debug!("Marking service {} for removal", service);
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

    // Create updated context with merged accounts for privilege service accumulation
    let updated_context = StateContext {
        accounts: accounts.clone(),
        privileges: context.privileges.clone(),
        validators: context.validators.clone(),
        authorization: context.authorization.clone(),
    };

    // get next context
    //
    // TODO: use a local task pool for spawning this calculation.

    // Accumulate privilege services - they should be able to run even without explicit accounts
    // as they are system services that may have implicit accounts or special handling
    let bless_result = self::once::<V>(
        updated_context.clone(),
        reports,
        table,
        context.privileges.bless,
    );

    let designate_result = self::once::<V>(
        updated_context.clone(),
        reports,
        table,
        context.privileges.designate,
    );

    let assign_result = self::once::<V>(
        updated_context.clone(),
        reports,
        table,
        context.privileges.assign,
    );

    for (id, privilege_account) in bless_result.context.accounts.iter() {
        tracing::debug!(
            "Processing bless service account {}, existing storage: {}, privilege storage: {}",
            id,
            accounts.get(id).map(|a| a.storage.len()).unwrap_or(0),
            privilege_account.storage.len()
        );
        if let Some(existing_account) = accounts.get_mut(id) {
            // Merge storage: preserve existing entries and add new ones from privilege account
            for (key, value) in &privilege_account.storage {
                existing_account.storage.insert(key.clone(), value.clone());
            }
            // Update other account fields if needed
            if privilege_account.balance != existing_account.balance {
                existing_account.balance = privilege_account.balance;
            }
        } else {
            accounts.insert(*id, privilege_account.clone());
        }
    }

    for (id, privilege_account) in designate_result.context.accounts.iter() {
        if let Some(existing_account) = accounts.get_mut(id) {
            // Merge storage: preserve existing entries and add new ones from privilege account
            for (key, value) in &privilege_account.storage {
                existing_account.storage.insert(key.clone(), value.clone());
            }
            // Update other account fields if needed
            if privilege_account.balance != existing_account.balance {
                existing_account.balance = privilege_account.balance;
            }
        } else {
            accounts.insert(*id, privilege_account.clone());
        }
    }

    for (id, privilege_account) in assign_result.context.accounts.iter() {
        if let Some(existing_account) = accounts.get_mut(id) {
            // Merge storage: preserve existing entries and add new ones from privilege account
            for (key, value) in &privilege_account.storage {
                existing_account.storage.insert(key.clone(), value.clone());
            }
            // Update other account fields if needed
            if privilege_account.balance != existing_account.balance {
                existing_account.balance = privilege_account.balance;
            }
        } else {
            accounts.insert(*id, privilege_account.clone());
        }
    }

    // accumulate gas from privilege services
    gas.insert(context.privileges.bless, bless_result.gas);
    transfers.extend(bless_result.transfers.iter().cloned());
    if let Some(hash) = bless_result.hash {
        pairings.insert(context.privileges.bless, hash);
    }

    gas.insert(context.privileges.designate, designate_result.gas);
    transfers.extend(designate_result.transfers.iter().cloned());
    if let Some(hash) = designate_result.hash {
        pairings.insert(context.privileges.designate, hash);
    }

    gas.insert(context.privileges.assign, assign_result.gas);
    transfers.extend(assign_result.transfers.iter().cloned());
    if let Some(hash) = assign_result.hash {
        pairings.insert(context.privileges.assign, hash);
    }

    tracing::debug!(
        "Final accumulated result - accounts: {:?}",
        accounts
            .iter()
            .map(|(id, acc)| (*id, acc.storage.len()))
            .collect::<Vec<_>>()
    );

    Accumulated {
        accumulated: reports.len(),
        context: StateContext {
            accounts,
            privileges: context.privileges.clone(),
            validators: context.validators.clone(),
            authorization: context.authorization.clone(),
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
            .flat_map(|r| &r.results)
            .filter(|result| result.service_id == service)
            .map(|result| result.accumulate_gas)
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
