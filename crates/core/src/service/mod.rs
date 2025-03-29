//! Service module

use crate::{Gas, ServiceId};
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;
pub use {
    account::{ServiceAccount, ServiceAccountData, ServiceAccountDataJson, ServiceAccountState},
    accumulate::{
        AccumulatedQueue, AlwaysAccumulateMapItem, Privileges, PrivilegesJson, ReadyQueue,
        ReadyRecord, ReadyRecordJson,
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

/// The privileged service indices (χ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct ServiceIndex {
    /// The manager of the service index (m)
    pub manager: u32,

    /// The authorized service indices (a)
    pub authorized: u32,

    /// index of the validator keys and metadata to be drawn
    /// from next (t)
    pub validator: u32,

    /// indices of services which automatically accumulate
    /// in each block together with a basic amount of gas with
    /// which each accumulates.
    pub gas: BTreeMap<u32, Gas>,
}

/// Represents a service item.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceItem {
    /// The id of the service item
    pub id: ServiceId,

    /// The info of the service item
    #[json(nested)]
    pub data: ServiceAccountData,
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
