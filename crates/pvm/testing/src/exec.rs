//! Execution API of JAM VM

use crate::Jam;
use anyhow::Result;
use podec::Encode;
use pvm::{Accumulated, Invocation, Reason};
use pvmi::Interpreter;
use score::{
    service::{Privileges, ServiceAccount, WorkExecResult, WorkPackage, WorkReport},
    state::account,
    vm::AccumulateState,
    ServiceId,
};
use std::collections::BTreeMap;
use worker::Worker;

/// The result of an execution
#[derive(Debug, Default)]
pub struct ExecutionInfo {
    /// The refine gas used
    pub refine_gas: u64,

    /// The accumulate gas used
    pub accumulate_gas: u64,

    /// The account changes
    pub accounts: BTreeMap<ServiceId, ServiceAccount>,
}

impl ExecutionInfo {
    /// Create a new execution info
    pub fn new(acc: Vec<Accumulated<BTreeMap<ServiceId, ServiceAccount>>>) -> Self {
        let mut info = Self::default();
        for acc in acc.iter() {
            info.accumulate_gas += acc.gas;

            // FIXME: need to merge account data
            info.accounts = acc.context.accounts.clone();
        }

        info
    }

    /// Get a storage of an account
    pub fn get_storage<V: podec::Decode>(&self, service: ServiceId, key: &[u8]) -> Option<V> {
        let account = self.accounts.get(&service)?;
        let key = account::storage(service, &key.encode());
        let encoded = account.storage.get(key.as_ref())?;
        V::decode(&mut &encoded[..]).ok()
    }
}

impl Jam {
    /// Execute a work item directly
    ///
    /// TODO: introduce better execution result
    pub fn execute(&mut self, service: ServiceId, payload: Vec<u8>) -> Result<ExecutionInfo> {
        let package = self.send(service, payload)?;
        let report = self.refine(&package)?;
        Ok(ExecutionInfo::new(self.accumulate(&report)?))
    }

    /// Authorize the work package
    pub fn authorize(&mut self, work: &WorkPackage, core_idx: u16) -> Result<pvm::Executed> {
        Ok(Interpreter::is_authorized(
            work,
            core_idx,
            &mut self.chain.accounts,
            self.chain.best.slot,
        ))
    }

    /// Refine the work package
    ///
    /// NOTE: run refine for all work items
    pub fn refine(&mut self, work: &WorkPackage) -> Result<WorkReport> {
        let mut worker = Worker::default();
        worker.refine::<_, Interpreter>(work, &mut self.chain.accounts, 0)?;

        // verify the work results
        let report = worker.report;
        for (index, result) in report.results.iter().enumerate() {
            if !matches!(result.result, WorkExecResult::Ok(_)) {
                return Err(anyhow::anyhow!(
                    "work item {index} refine failed: {:?}",
                    result.result
                ));
            }
        }
        Ok(report)
    }

    /// Accumulate the work package
    ///
    /// 1. convert work package to work report
    /// 2. run accumulate for all work items
    /// 3. return the accumulated result
    pub fn accumulate(
        &mut self,
        report: &WorkReport,
    ) -> Result<Vec<Accumulated<BTreeMap<ServiceId, ServiceAccount>>>> {
        let accounts = self.chain.accounts.clone();
        let mut state = AccumulateState {
            accounts,
            validators: vec![],
            authorization: Default::default(),
            privileges: Privileges::default(),
        };

        let mut batch = BTreeMap::new();
        for result in report.results.iter() {
            *batch.entry(result.service_id).or_insert(0) += result.accumulate_gas;
        }

        // TODO: merge account data
        let mut result = Vec::new();
        for (service_id, gas) in batch.iter() {
            let accumulated = Interpreter::accumulate(
                state,
                self.chain.best.slot,
                *service_id,
                *gas,
                report.operands(*service_id),
                self.chain.entropy[1],
            );

            if accumulated.reason.is_err() && accumulated.reason != Reason::Halt {
                anyhow::bail!("accumulate failed: {:?}", accumulated.reason);
            }
            state = accumulated.context.clone();
            result.push(accumulated);
        }
        self.chain.accounts = state.accounts;
        Ok(result)
    }
}
