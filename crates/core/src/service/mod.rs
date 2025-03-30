//! Service module

use crate::Gas;
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    account::{
        ServiceAccount, ServiceAccountData, ServiceAccountDataJson, ServiceAccountState,
        ServiceItem, ServiceItemJson, ServicePreimage, ServicePreimageJson,
    },
    accumulate::{
        AccumulatedQueue, Privileges, PrivilegesJson, ReadyQueue, ReadyReport, ReadyReportJson,
    },
    refine::{RefineContext, RefineContextJson, RefineLoad, RefineLoadJson},
    report::{ReportedWorkPackage, ReportedWorkPackageJson, WorkReport, WorkReportJson},
    result::{WorkExecResult, WorkExecResultJson, WorkResult, WorkResultJson},
    work::{
        WorkItem, WorkItemJson, WorkPackage, WorkPackageJson, WorkPackageSpec, WorkPackageSpecJson,
    },
};

mod account;
mod accumulate;
mod refine;
mod report;
mod result;
mod work;

/// The availability assignments item
pub type AvailabilityAssignmentsItem = Option<AvailabilityAssignment>;

/// The availability assignments
pub type AvailabilityAssignments = [AvailabilityAssignmentsItem; crate::CORES_COUNT];

/// The availability assignment
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct AvailabilityAssignment {
    /// The report
    #[json(nested)]
    pub report: WorkReport,

    /// The timeout
    pub timeout: u32,
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
