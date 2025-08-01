//! Network provider trait for work package and report operations

use score::{
    extrinsic::ReportGuarantee,
    service::{WorkPackage, WorkReport},
    Ed25519Public, OpaqueHash, TimeSlot, ValidatorIndex,
};

/// Network provider trait for work package and report coordination
///
/// This trait combines network operations with validator functionality,
/// since Network<C> has direct access to Runtime<C> and all validator logic.
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

    /// Validate a work report according to Gray Paper specifications
    /// Uses GuaranteeValidator logic from runtime
    async fn validate_work_report(
        &self,
        report: &WorkReport,
        slot: TimeSlot,
    ) -> anyhow::Result<bool>;

    /// Get validator grid neighbors for efficient communication
    /// Uses Grid logic from runtime for neighbor discovery
    async fn neighbors(&self, validator: Ed25519Public) -> anyhow::Result<Vec<Ed25519Public>>;

    /// Get the local validator's index in the validator set
    async fn validator_index(&self) -> anyhow::Result<ValidatorIndex>;

    /// Create a complete report guarantee with signatures
    /// Combines work report with validator signatures for on-chain submission
    async fn guarantee(
        &self,
        report: WorkReport,
        slot: TimeSlot,
    ) -> anyhow::Result<ReportGuarantee>;

    /// Check if local validator is assigned to a specific core at given timeslot
    async fn is_core_guarantor(&self, core_idx: usize, timeslot: u32) -> anyhow::Result<bool>;

    /// Get the current validator set for signature verification
    async fn current_validators(&self) -> anyhow::Result<Vec<Ed25519Public>>;
}
