//! Refine related types

use crate::{OpaqueHash, ServiceId};
use serde::{Deserialize, Serialize};

/// Refine parameters
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct RefineParams {
    /// (c) the core index
    #[serde(with = "codec::compact")]
    pub core: u16,

    /// (i) the work item index
    #[serde(with = "codec::compact")]
    pub index: u16,

    /// (w_s) the service id
    #[serde(with = "codec::compact")]
    pub id: ServiceId,

    /// (y) the payload
    pub payload: Vec<u8>,

    /// (p) the work package hash
    pub package: OpaqueHash,
}
