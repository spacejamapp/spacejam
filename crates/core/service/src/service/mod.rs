//! Service types of SpaceJam

use crate::{BTreeMap, Gas, ServiceId};
use serde::{Deserialize, Serialize};
pub use {
    account::ServiceAccount,
    refine::{RefineContext, RefineLoad},
    result::{WorkExecResult, WorkResult},
    work::{ExtrinsicSpec, ImportSpec, WorkItem, WorkPackage, WorkPackageSpec},
};

#[cfg(feature = "json")]
use {crate::Vec, spacejson::Json};

#[cfg(feature = "json")]
pub use {
    refine::{RefineContextJson, RefineLoadJson},
    result::{WorkExecResultJson, WorkResultJson},
    work::{ExtrinsicSpecJson, ImportSpecJson, WorkPackageSpecJson},
};

pub mod account;
pub mod refine;
pub mod result;
pub mod work;

/// The privileged service indices (χ)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct Privileges {
    /// The bless service id (χm)
    pub bless: ServiceId,

    /// The assign service id (χa)
    #[cfg_attr(feature = "json", json(Vec<ServiceId>))]
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "json", derive(Json))]
pub struct GasLimit {
    /// The minimum gas in order to execute the accumulate
    /// entry-point of the service code (g)
    #[serde(alias = "min_memo_gas")]
    pub accumulate: Gas,

    /// The minimum required for the on transfer entry-point (m)
    #[serde(alias = "min_item_gas")]
    pub transfer: Gas,
}
