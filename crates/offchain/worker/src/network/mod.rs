//! Network layer for the worker

pub use provider::NetworkProvider;
use score::{
    extrinsic::{ReportGuarantee, ValidatorSignature},
    service::{WorkPackage, WorkReport},
    Ed25519Public, OpaqueHash, TimeSlot, ValidatorIndex,
};

mod provider;

/// Dummy network provider for testing
impl NetworkProvider for () {
    // Core Network Operations

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

    // Validator Operations (dummy implementations for testing)

    async fn validate_work_report(
        &self,
        _report: &WorkReport,
        _slot: TimeSlot,
    ) -> anyhow::Result<bool> {
        Ok(true) // Always validate successfully in tests
    }

    async fn neighbors(&self, _validator: Ed25519Public) -> anyhow::Result<Vec<Ed25519Public>> {
        Ok(vec![])
    }

    async fn validator_index(&self) -> anyhow::Result<ValidatorIndex> {
        Ok(0)
    }

    async fn guarantee(
        &self,
        report: WorkReport,
        slot: TimeSlot,
    ) -> anyhow::Result<ReportGuarantee> {
        Ok(ReportGuarantee {
            report,
            slot,
            signatures: vec![ValidatorSignature {
                validator_index: 0,
                signature: [0u8; 64],
            }],
        })
    }

    async fn is_core_guarantor(&self, _core_idx: usize, _timeslot: u32) -> anyhow::Result<bool> {
        Ok(true) // Always return true for testing
    }

    async fn current_validators(&self) -> anyhow::Result<Vec<Ed25519Public>> {
        Ok(vec![[0u8; 32]]) // Return dummy validator for testing
    }
}
