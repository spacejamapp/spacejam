//! The state of the reporting portion of the protocol.

use score::{
    block::{BlockInfo, BlockInfoJson},
    extrinsic::{GuaranteesExtrinsic, ReportGuaranteeJson},
    service::{ServiceInfo, ServiceInfoJson},
    validator::{ValidatorDataJson, ValidatorsData},
    work::{AvailabilityAssignmentJson, AvailabilityAssignments},
    Ed25519Public, EntropyBuffer, OpaqueHash, ServiceId, TimeSlot, CORES_COUNT,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceItem {
    pub id: ServiceId,
    #[json(nested)]
    pub info: ServiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct State {
    /// [ρ‡] Intermediate pending reports after that any work report judged as
    /// uncertain or invalid has been removed from it (ϱ†), and the availability
    /// assurances are processed. Mutated to ϱ'.
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub avail_assignments: AvailabilityAssignments,

    /// [κ'] Posterior active validators.
    #[json(Vec<ValidatorDataJson>)]
    pub curr_validators: ValidatorsData,

    /// [λ'] Posterior previous validators.
    #[json(Vec<ValidatorDataJson>)]
    pub prev_validators: ValidatorsData,

    /// [η'] Posterior entropy buffer.
    #[json(Vec<String>)]
    pub entropy: EntropyBuffer,

    /// [ψ'_o] Posterior offenders.
    #[json(Vec<String>)]
    pub offenders: Vec<Ed25519Public>,

    /// [β] Recent blocks.
    #[json(Vec<BlockInfoJson>)]
    pub recent_blocks: Vec<BlockInfo>,

    /// Authorization pools.
    #[json(Vec<Vec<String>>)]
    pub auth_pools: [Vec<OpaqueHash>; CORES_COUNT],

    /// [δ] Encoded services dictionary. Refer to T(σ) in Appendix D.
    #[json(nested)]
    pub services: Vec<ServiceItem>,
}

/// A reported work package with its dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ReportedPackage {
    #[json(hex)]
    pub work_package_hash: OpaqueHash,
    #[json(hex)]
    pub segment_tree_root: OpaqueHash,
}

/// Input of the reporting module.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct Input {
    pub slot: TimeSlot,
    #[json(Vec<ReportGuaranteeJson>)]
    pub guarantees: GuaranteesExtrinsic,
}

/// Output of the reporting module.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Output {
    #[json(nested)]
    pub reported: Vec<ReportedPackage>,
    #[json(Vec<String>)]
    pub reporters: Vec<Ed25519Public>,
}
