//! Core statistics

use crate::Gas;
use codec::Compact;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a core record.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Default)]
pub struct CoreActivityRecord {
    /// Amount of bytes which are placed into either Audits or Segments DA.
    /// This includes the work-bundle (including all extrinsics and imports) as well as all
    /// (exported) segments.
    #[json(compact)]
    pub da_load: Compact<u32>,

    /// Number of validators which formed super-majority for assurance.
    #[json(compact)]
    pub popularity: Compact<u16>,

    /// Number of segments imported from DA made by core for reported work.
    #[json(compact)]
    pub imports: Compact<u16>,

    /// Number of segments exported into DA made by core for reported work.
    #[json(compact)]
    pub exports: Compact<u16>,

    /// Total size of extrinsics used by core for reported work.
    #[json(compact)]
    pub extrinsic_size: Compact<u32>,

    /// Total number of extrinsics used by core for reported work.
    #[json(compact)]
    pub extrinsic_count: Compact<u16>,

    /// The work-bundle size. This is the size of data being placed into Audits DA by the core.
    #[json(compact)]
    pub bundle_size: Compact<u32>,

    /// Total gas consumed by core for reported work. Includes all refinement and authorizations.
    #[json(compact)]
    pub gas_used: Compact<Gas>,
}
