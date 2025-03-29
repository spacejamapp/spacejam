//! Service module

use crate::{BeefyRoot, Gas, HeaderHash, OpaqueHash, ServiceId, StateRoot, TimeSlot};
use codec::Compact;
use serde::{Deserialize, Serialize};
use spacejson::Json;
use std::collections::BTreeMap;
pub use {
    report::{WorkExecResult, WorkExecResultJson, WorkReport, WorkReportJson, WorkResult},
    work::{ReportedWorkPackage, ReportedWorkPackageJson, WorkItem, WorkPackage},
};

mod report;
mod work;

/// The service accounts (δ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct ServiceAccount {
    /// storage of the service account (s)
    pub storage: BTreeMap<OpaqueHash, Vec<u8>>,

    /// The preimage of the service account (p)
    pub preimage: BTreeMap<OpaqueHash, Vec<u8>>,

    /// Preimage lookup dictionary (l)
    pub lookup: BTreeMap<(OpaqueHash, u32), [TimeSlot; 3]>,

    /// The code hash of the service account (c)
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The gas limits of the service account (g) and (m)
    #[serde(flatten)]
    pub gas: GasLimit,
}

impl ServiceAccount {
    /// The number of items in storage
    pub fn items(&self) -> u32 {
        2 * self.lookup.len() as u32 + self.storage.len() as u32
    }

    /// total number of octets used in storage
    pub fn total(&self) -> u64 {
        self.lookup
            .iter()
            .map(|((_, z), _)| 81 + *z as u64)
            .chain(self.storage.values().map(|x| 32 + x.len() as u64))
            .sum::<u64>()
    }

    /// The state of the service account
    pub fn state(&self) -> ServiceAccountState {
        ServiceAccountState {
            code: self.code,
            balance: self.balance,
            gas: self.gas.clone(),
            total: self.total(),
            items: self.items(),
        }
    }
}

/// The state of the service account
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Json)]
pub struct ServiceAccountState {
    /// The code hash of the service account (c)
    #[json(hex)]
    #[serde(alias = "code_hash")]
    pub code: OpaqueHash,

    /// The balance of the service account (b)
    pub balance: u64,

    /// The gas limits of the service account (g) and (m)
    #[serde(flatten)]
    #[json(nested)]
    pub gas: GasLimit,

    /// The total number of octets used in storage (o)
    #[serde(alias = "bytes")]
    pub total: u64,

    /// The number of items in storage (i)
    pub items: u32,
}

/// The gas limits of the service account
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default, Json)]
pub struct GasLimit {
    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    #[serde(alias = "min_memo_gas")]
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    #[serde(alias = "min_item_gas")]
    pub transfer: Gas,
}

/// The privileged service indices (χ)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct ServiceIndex {
    /// The manager of the service index (m)
    pub manager: u32,

    /// The authorized service indices (a)
    pub authorized: u32,

    /// index of the validator keys and metadata to be drawn
    /// from next (t)
    pub validator: u32,

    /// indices of services which automatically accumulate
    /// in each block together with a basic amount of gas with
    /// which each accumulates.
    pub gas: BTreeMap<u32, Gas>,
}

/// Represents a service item.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceItem {
    /// The id of the service item
    pub id: ServiceId,

    /// The info of the service item
    #[json(nested)]
    pub data: ServiceAccountData,
}

/// Represents the service account data.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct ServiceAccountData {
    /// The service account state
    #[json(nested)]
    pub service: ServiceAccountState,
}

/// Represents the RefineContext structure from ASN.1
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct RefineContext {
    #[json(hex)]
    pub anchor: HeaderHash,
    #[json(hex)]
    pub state_root: StateRoot,
    #[json(hex)]
    pub beefy_root: BeefyRoot,
    #[json(hex)]
    pub lookup_anchor: HeaderHash,
    pub lookup_anchor_slot: TimeSlot,
    #[json(hex)]
    pub prerequisites: Vec<OpaqueHash>,
}

/// The refine load
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct RefineLoad {
    /// The gas used
    #[json(compact)]
    pub gas_used: Compact<u64>,

    /// The number of imports
    #[json(compact)]
    pub imports: Compact<u16>,

    /// The number of extrinsics
    #[json(compact)]
    pub extrinsic_count: Compact<u16>,

    /// The size of the extrinsics
    #[json(compact)]
    pub extrinsic_size: Compact<u32>,

    /// The number of exports
    #[json(compact)]
    pub exports: Compact<u16>,
}

#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct AvailabilityAssignment {
    #[json(nested)]
    pub report: WorkReport,
    pub timeout: u32,
}

pub type AvailabilityAssignmentsItem = Option<AvailabilityAssignment>;
pub type AvailabilityAssignments = [AvailabilityAssignmentsItem; crate::CORES_COUNT];
