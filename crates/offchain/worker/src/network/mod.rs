//! Network layer for the worker

mod provider;

pub use provider::NetworkProvider;
use score::{
    service::{WorkPackage, WorkReport},
    OpaqueHash,
};

/// Dummy network provider for testing
impl NetworkProvider for () {
    async fn core_assignments(&self, _timeslot: u32) -> anyhow::Result<Vec<Vec<u16>>> {
        Ok(vec![])
    }

    async fn guarantor_peers(
        &self,
        _core_idx: usize,
        _timeslot: u32,
    ) -> anyhow::Result<Vec<OpaqueHash>> {
        Ok(vec![])
    }

    async fn submit_work_package(
        &self,
        _package: WorkPackage,
        _core_idx: usize,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn share_with_guarantors(
        &self,
        _package: WorkPackage,
        _guarantors: &[OpaqueHash],
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn distribute_report(&self, _report: WorkReport) -> anyhow::Result<()> {
        Ok(())
    }

    async fn broadcast_to_validators(
        &self,
        _report: WorkReport,
        _targets: &[OpaqueHash],
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
