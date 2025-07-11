//! Refine related types

use crate::{OpaqueHash, ServiceId};
use serde::{Deserialize, Serialize};

/// Refine parameters
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct RefineParams {
    /// (i) the work item index
    pub index: usize,

    /// (w_s) the service id
    pub id: ServiceId,

    /// (y) the payload
    pub payload: Vec<u8>,

    /// (p) the work package hash
    pub package: OpaqueHash,
}
