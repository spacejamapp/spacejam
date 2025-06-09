//! Core statistics

use crate::Gas;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a core record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct CoreActivityRecord {
    /// (d) Amount of bytes which are placed into either Audits or Segments DA.
    /// This includes the work-bundle (including all extrinsics and imports) as well as all
    /// (exported) segments.
    #[serde(with = "codec::compact")]
    pub da_load: u32,

    /// (p) Number of validators which formed super-majority for assurance.
    #[serde(with = "codec::compact")]
    pub popularity: u16,

    /// (i) Number of segments imported from DA made by core for reported work.
    #[serde(with = "codec::compact")]
    pub imports: u16,

    /// (e) Number of segments exported into DA made by core for reported work.
    #[serde(with = "codec::compact")]
    pub exports: u16,

    /// (z) Total size of extrinsics used by core for reported work.
    #[serde(with = "codec::compact")]
    pub extrinsic_size: u32,

    /// (x) Total number of extrinsics used by core for reported work.
    #[serde(with = "codec::compact")]
    pub extrinsic_count: u16,

    /// (b) The work-bundle size. This is the size of data being placed into Audits DA by the core.
    #[serde(with = "codec::compact")]
    pub bundle_size: u32,

    /// (u) Total gas consumed by core for reported work. Includes all refinement and authorizations.
    #[serde(with = "codec::compact")]
    pub gas_used: Gas,
}
