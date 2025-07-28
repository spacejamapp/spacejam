//! Execution API of JAM VM

use std::collections::BTreeMap;

use crate::Jam;
use anyhow::Result;
use pvm::{Invocation, Reason};
use pvmi::Interpreter;
use score::{
    service::{Privileges, WorkExecResult, WorkPackage, WorkReport},
    vm::AccumulateState,
};
use worker::Worker;

impl Jam {
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
        let mut worker = Worker::new();
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
    pub fn accumulate(&mut self, report: &WorkReport) -> Result<()> {
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

        for (service_id, gas) in batch.iter() {
            let accumulated = Interpreter::accumulate(
                state,
                self.chain.best.slot,
                *service_id,
                *gas,
                report.operands(*service_id),
                self.chain.entropy[1],
            );

            state = accumulated.context;
            if accumulated.reason.is_err() && accumulated.reason != Reason::Halt {
                anyhow::bail!("accumulate failed: {:?}", accumulated.reason);
            }
        }

        self.chain.accounts = state.accounts;
        Ok(())
    }
}
