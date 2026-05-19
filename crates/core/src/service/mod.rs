//! Service module

use crate::{CORES_COUNT, WorkPackageHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    report::{
        ReadyReport, ReadyReportJson, ReportedWorkPackage, ReportedWorkPackageJson, WorkReport,
        WorkReportJson,
    },
    service::service::{
        Privileges, PrivilegesJson,
        account::{ServiceAccount, ServiceInfo, ServiceInfoJson},
        refine::{RefineContext, RefineContextJson, RefineLoad, RefineLoadJson},
        result::{
            Executed, Refined, WorkDigest, WorkDigestJson, WorkExecResult, WorkExecResultJson,
        },
        work::{
            ExtrinsicSpec, ImportSpec, WorkItem, WorkItemJson, WorkPackage, WorkPackageJson,
            WorkPackageSpec, WorkPackageSpecJson,
        },
    },
    validate::PackageValidation,
};

mod report;
mod validate;

/// The ready queue (θ)
pub type ReadyQueue = crate::Array<Vec<ReadyReport>, { crate::EPOCH_LENGTH as usize }>;

/// The accumulated queue (ξ)
pub type AccumulatedQueue = crate::Array<Vec<WorkPackageHash>, { crate::EPOCH_LENGTH as usize }>;

/// The availability assignments (ρ)
pub type AvailabilityAssignments =
    crate::Array<Option<AvailabilityAssignment>, CORES_COUNT>;

/// The availability assignment
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct AvailabilityAssignment {
    /// The report
    #[json(nested)]
    pub report: WorkReport,

    /// The timeout
    pub timeout: u32,
}
