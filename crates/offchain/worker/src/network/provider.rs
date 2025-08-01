//! Network provider trait for work package and report operations

use score::{
    service::{WorkPackage, WorkReport},
    OpaqueHash,
};

/// Network provider trait for work package and report coordination
#[allow(async_fn_in_trait)]
pub trait NetworkProvider: Send + Sync {
    /// Get core assignments for a specific timeslot
    async fn core_assignments(&self, timeslot: u32) -> anyhow::Result<Vec<Vec<u16>>>;

    /// Get peer identifiers for guarantors of a specific core
    async fn guarantor_peers(
        &self,
        core_idx: usize,
        timeslot: u32,
    ) -> anyhow::Result<Vec<OpaqueHash>>;

    /// Submit a work package to guarantors (CE133)
    async fn submit_work_package(
        &self,
        package: WorkPackage,
        core_idx: usize,
    ) -> anyhow::Result<()>;

    /// Share work package with other guarantors (CE134)
    async fn share_with_guarantors(
        &self,
        package: WorkPackage,
        guarantors: &[OpaqueHash],
    ) -> anyhow::Result<()>;

    /// Distribute work report to validators (CE135)
    async fn distribute_report(&self, report: WorkReport) -> anyhow::Result<()>;

    /// Broadcast work report to specific validator targets
    async fn broadcast_to_validators(
        &self,
        report: WorkReport,
        targets: &[OpaqueHash],
    ) -> anyhow::Result<()>;
}
