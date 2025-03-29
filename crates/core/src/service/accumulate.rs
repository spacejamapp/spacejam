//! Accumulate types

use crate::{
    service::{WorkReport, WorkReportJson},
    Gas, ServiceId, WorkPackageHash,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// The ready queue
pub type ReadyQueue = [Vec<ReadyRecord>; crate::EPOCH_LENGTH as usize];

/// The accumulated queue
pub type AccumulatedQueue = [Vec<WorkPackageHash>; crate::EPOCH_LENGTH as usize];

/// The privileges
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Privileges {
    /// The bless service id
    pub bless: ServiceId,

    /// The assign service id
    pub assign: ServiceId,

    /// The designate service id
    pub designate: ServiceId,

    /// The always accumulate service ids
    #[json(nested)]
    pub always_acc: Vec<AlwaysAccumulateMapItem>,
}

/// The always accumulate map item
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct AlwaysAccumulateMapItem {
    /// The service id
    pub service: ServiceId,

    /// The gas
    pub gas: Gas,
}

/// The ready record
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ReadyRecord {
    /// The report
    #[json(nested)]
    pub report: WorkReport,

    /// The dependencies
    #[json(Vec<String>)]
    pub dependencies: Vec<WorkPackageHash>,
}
