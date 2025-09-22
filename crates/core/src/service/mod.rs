//! Service module

use crate::WorkPackageHash;
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    jcore::service::{
        Privileges, PrivilegesJson,
        account::{ServiceAccount, ServiceInfo, ServiceInfoJson},
        refine::{RefineContext, RefineContextJson, RefineLoad, RefineLoadJson},
        result::{WorkExecResult, WorkExecResultJson, WorkResult, WorkResultJson},
        work::{
            ExtrinsicSpec, ImportSpec, WorkItem, WorkItemJson, WorkPackage, WorkPackageJson,
            WorkPackageSpec, WorkPackageSpecJson,
        },
    },
    report::{
        ReadyReport, ReadyReportJson, ReportedWorkPackage, ReportedWorkPackageJson, WorkReport,
        WorkReportJson,
    },
    validate::PackageValidation,
};

mod report;
mod validate;

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
