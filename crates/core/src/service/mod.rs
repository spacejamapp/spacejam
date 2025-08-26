//! Service module

use crate::{Gas, ServiceId, WorkPackageHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;
pub use {
    account::{ServiceAccount, ServiceInfo, ServiceInfoJson},
    refine::{RefineContext, RefineContextJson, RefineLoad, RefineLoadJson},
    report::{
        ReadyReport, ReadyReportJson, ReportedWorkPackage, ReportedWorkPackageJson, WorkReport,
        WorkReportJson,
    },
    result::{WorkExecResult, WorkExecResultJson, WorkResult, WorkResultJson},
    validate::PackageValidation,
    work::{
        ExtrinsicSpec, ImportSpec, WorkItem, WorkItemJson, WorkPackage, WorkPackageJson,
        WorkPackageSpec, WorkPackageSpecJson,
    },
};

mod account;
mod refine;
mod report;
mod result;
mod validate;
mod work;

/// The ready queue (θ)
pub type ReadyQueue = [Vec<ReadyReport>; crate::EPOCH_LENGTH as usize];

/// The accumulated queue (ξ)
pub type AccumulatedQueue = [Vec<WorkPackageHash>; crate::EPOCH_LENGTH as usize];

/// The availability assignments (ρ)
pub type AvailabilityAssignments = [Option<AvailabilityAssignment>; crate::CORES_COUNT];

/// The availability assignment
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct AvailabilityAssignment {
    /// The report
    #[json(nested)]
    pub report: WorkReport,

    /// The timeout
    pub timeout: u32,
}

/// The privileged service indices (χ)
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq, Default)]
pub struct Privileges {
    /// The bless service id (χm)
    pub bless: ServiceId,

    /// The assign service id (χa)
    #[json(Vec<ServiceId>)]
    pub assign: [ServiceId; crate::CORES_COUNT],

    /// The designate service id (χv)
    pub designate: ServiceId,

    /// The always accumulate service ids (χg)
    pub always_acc: BTreeMap<ServiceId, Gas>,
}

impl Privileges {
    /// Get the gas limit from the privileges
    pub fn gas_limit(&self) -> Gas {
        (crate::GAS_ACC * crate::CORES_COUNT as u64 + self.always_acc.values().sum::<u64>())
            .max(crate::GAS_ALL_ACC)
    }
}

/// The gas limits of the service account
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default, Json)]
pub struct GasLimit {
    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    #[serde(alias = "min_memo_gas")]
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    #[serde(alias = "min_item_gas")]
    pub transfer: Gas,
}
