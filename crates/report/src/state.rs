//! The state of the reporting portion of the protocol.

use score::{
    block::history::{BlockInfo, BlockInfoJson},
    misc::{
        Ed25519Public, EntropyBuffer, OpaqueHash, ServiceId, ServiceInfo, ServiceInfoJson,
        ValidatorDataJson, ValidatorsData,
    },
    work::{AvailabilityAssignmentJson, AvailabilityAssignments},
    CORES_COUNT,
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct ServiceItem {
    pub id: ServiceId,
    #[json(nested)]
    pub info: ServiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Json)]
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
    pub auth_pools: [Vec<OpaqueHash>; CORES_COUNT as usize],

    /// [δ] Encoded services dictionary. Refer to T(σ) in Appendix D.
    #[json(nested)]
    pub services: Vec<ServiceItem>,
}

/// A reported work package with its dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct ReportedPackage {
    #[json(hex)]
    work_package_hash: OpaqueHash,
    #[json(hex)]
    segment_tree_root: OpaqueHash,
}

/// Output of the reporting module.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct Output {
    #[json(nested)]
    pub reported: Vec<ReportedPackage>,
    #[json(Vec<String>)]
    pub reporters: Vec<Ed25519Public>,
}
