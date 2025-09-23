//! Service types of SpaceJam

use crate::{BTreeMap, Gas, ServiceId, Vec};
use serde::{Deserialize, Serialize};
use spacejson::Json;
pub use {
    account::ServiceAccount,
    refine::{RefineContext, RefineContextJson, RefineLoad, RefineLoadJson},
    result::{WorkExecResult, WorkExecResultJson, WorkResult, WorkResultJson},
    work::{ExtrinsicSpec, ImportSpec, WorkItem, WorkPackage, WorkPackageSpec},
};

pub mod account;
pub mod refine;
pub mod result;
pub mod work;

/// The privileged service indices (χ)
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq, Default)]
pub struct Privileges {
    /// The bless service id (χm)
    pub bless: ServiceId,

    /// The assign service id (χa)
    #[json(Vec<ServiceId>)]
    pub assign: [ServiceId; crate::CORES_COUNT],

    /// The designate service id (χv)
    pub designate: ServiceId,

    /// The always accumulate service ids (χg)
    pub always_acc: BTreeMap<ServiceId, Gas>,
}

impl Privileges {
    /// Get the gas limit from the privileges
    pub fn gas_limit(&self) -> Gas {
        (crate::GAS_ACC * crate::CORES_COUNT as u64 + self.always_acc.values().sum::<u64>())
            .max(crate::GAS_ALL_ACC)
    }
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
