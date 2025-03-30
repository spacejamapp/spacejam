//! Accumulate types

use std::collections::BTreeMap;

use crate::{
    service::{WorkReport, WorkReportJson},
    Gas, ServiceId, WorkPackageHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// The ready queue (θ)
pub type ReadyQueue = [Vec<ReadyReport>; crate::EPOCH_LENGTH as usize];

/// The accumulated queue (ξ)
pub type AccumulatedQueue = [Vec<WorkPackageHash>; crate::EPOCH_LENGTH as usize];

/// The privileged service indices (χ)
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq, Default)]
pub struct Privileges {
    /// The bless service id (χm)
    pub bless: ServiceId,

    /// The designate service id (χv)
    pub designate: ServiceId,

    /// The assign service id (χa)
    pub assign: ServiceId,

    /// The always accumulate service ids (χg)
    pub always_acc: BTreeMap<ServiceId, Gas>,
}

/// The ready record
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ReadyReport {
    /// The report
    #[json(nested)]
    pub report: WorkReport,

    /// The dependencies
    #[json(Vec<String>)]
    pub dependencies: Vec<WorkPackageHash>,
}
