//! Execution of work reports

use pvm::{AccumulateResult, Pvm};
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
    context: StateContext,
    gas_table: &BTreeMap<ServiceId, Gas>,
    timeslot: TimeSlot,
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

    let mut accumulated =
        self::parallel::<V>(context.clone(), &reports[..index], gas_table, timeslot);
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
    context: StateContext,
    reports: &[WorkReport],
    table: &BTreeMap<ServiceId, Gas>,
    timeslot: TimeSlot,
) -> Accumulated {
    // TODO: use a local task pool for spawning this calculation.
    let mut services: BTreeSet<ServiceId> = table.keys().cloned().collect();
    for report in reports {
        for result in &report.results {
            services.insert(result.service_id);
        }
    }

    let results: BTreeMap<ServiceId, AccumulateResult> = services
        .iter()
        .map(|service| {
            let result = self::once::<V>(context.clone(), reports, table, *service, timeslot);
            (*service, result)
        })
        .collect();

    // According to the specification in accumulation.tex:
    // d' = P((d ∪ n) ∖ m, p)
    // where:
    // n = ⋃_{s ∈ s}({(Δ₁(o, w, f, s)_o)_d ∖ keys{d ∖ {s}}})
    // m = ⋃_{s ∈ s}(keys{d} ∖ keys{(Δ₁(o, w, f, s)_o)_d})
    let original_accounts = &context.accounts;
    let mut new_accounts = BTreeSet::new(); // p
    let mut removed_accounts = BTreeSet::new(); // m
    let mut gas = BTreeMap::new();
    let mut transfers = vec![];
    let mut pairings = BTreeMap::new();

    // Process each service result according to the specification
    for (service_id, result) in results.iter() {
        // Calculate new accounts for this service (accounts not in original except the service itself)
        for account_id in result.context.accounts.keys() {
            if !original_accounts.contains_key(account_id)
                || *account_id == *service_id && !original_accounts.contains_key(service_id)
            {
                new_accounts.insert(*account_id);
            }
        }

        // Calculate removed accounts for this service (original accounts not in result)
        for account_id in original_accounts.keys() {
            if !result.context.accounts.contains_key(account_id) {
                removed_accounts.insert(*account_id);
            }
        }

        // Collect other outputs
        gas.insert(*service_id, result.gas);
        transfers.extend(result.transfers.clone());
        if let Some(hash) = result.hash {
            pairings.insert(*service_id, hash);
        }
    }

    // Build the final account state according to: (d ∪ n) ∖ m
    let mut final_accounts = original_accounts.clone();

    // Add new accounts and update existing ones from service executions
    for (service_id, result) in results.iter() {
        for (account_id, account) in result.context.accounts.iter() {
            if new_accounts.contains(account_id)
                || (original_accounts.contains_key(account_id) && *account_id == *service_id)
            {
                // Only update accounts that are new or belong to the current service
                final_accounts.insert(*account_id, account.clone());
            }
        }
    }

    // Remove accounts that were removed by any service
    for account_id in removed_accounts {
        final_accounts.remove(&account_id);
    }

    // Create updated context for privilege service accumulation
    let updated_context = StateContext {
        accounts: final_accounts.clone(),
        privileges: context.privileges.clone(),
        validators: context.validators.clone(),
        authorization: context.authorization.clone(),
    };

    // Process privilege services (χₘ, χᵥ, χₐ)
    // These should be processed after regular services and can modify the final state
    let privilege_services = [
        context.privileges.bless,
        context.privileges.designate,
        context.privileges.assign,
    ];

    for &privilege_service in &privilege_services {
        let privilege_result = self::once::<V>(
            updated_context.clone(),
            reports,
            table,
            privilege_service,
            timeslot,
        );

        // For privilege services, we allow them to modify any account
        // but we still need to be careful about conflicts
        for (account_id, privilege_account) in privilege_result.context.accounts.iter() {
            if let Some(existing_account) = final_accounts.get_mut(account_id) {
                // Merge storage: privilege services can update existing accounts
                for (key, value) in &privilege_account.storage {
                    existing_account.storage.insert(key.clone(), value.clone());
                }
                // Update other account fields
                existing_account.balance = privilege_account.balance;
                existing_account.gas = privilege_account.gas.clone();
                existing_account.code = privilege_account.code;

                // Merge preimages and lookup tables
                for (hash, preimage) in &privilege_account.preimage {
                    existing_account.preimage.insert(*hash, preimage.clone());
                }
                for (lookup_key, slots) in &privilege_account.lookup {
                    existing_account.lookup.insert(*lookup_key, slots.clone());
                }
            } else {
                // New account created by privilege service
                final_accounts.insert(*account_id, privilege_account.clone());
            }
        }

        // Collect privilege service outputs
        gas.insert(privilege_service, privilege_result.gas);
        transfers.extend(privilege_result.transfers);
        if let Some(hash) = privilege_result.hash {
            pairings.insert(privilege_service, hash);
        }
    }

    Accumulated {
        accumulated: reports.len(),
        context: StateContext {
            accounts: final_accounts,
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
    timeslot: TimeSlot,
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
    V::accumulate(context, timeslot, service, gas, operands, [0; 32])
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
